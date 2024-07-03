use crate::error::{Error, Result};
use crate::{DataType, FieldCase, FieldInfo, MessagePath, Msg, Value};
use byteorder::{ReadBytesExt, LE};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::RegexBuilder;
use rustc_hash::FxHashMap;
use std::convert::TryInto;
use std::io::{self, Read};

type MyMap<K, V> = FxHashMap<K, V>;
type MessageValues = Vec<Value>;

#[derive(Clone, Debug)]
pub struct DynamicMsg {
    msg: Msg,
    dependencies: MyMap<MessagePath, Msg>,
}

/// Byte alignment for CDR version 1
const ALIGNMENT: usize = 8;

impl DynamicMsg {
    pub fn new(message_type: &str, message_definition: &str) -> Result<Self> {
        lazy_static! {
            static ref RE_DESCRIPTOR_MESSAGES_SPLITTER: regex::Regex = RegexBuilder::new("^=+$")
                .multi_line(true)
                .build()
                .expect("Invalid regex `^=+$`");
        }
        let mut message_bodies = RE_DESCRIPTOR_MESSAGES_SPLITTER.split(message_definition);
        let message_src = message_bodies
            .next()
            .ok_or(Error::BadMessageContent(format!(
                "Message definition for {} is missing main message body",
                message_type
            )))?;
        let msg = Self::parse_msg(message_type, message_src)?;
        let mut dependencies = MyMap::default();
        for message_body in message_bodies {
            let dependency = Self::parse_dependency(message_body)?;
            dependencies.insert(dependency.path().clone(), dependency);
        }

        Ok(DynamicMsg { msg, dependencies })
    }

    pub fn msg(&self) -> &Msg {
        &self.msg
    }

    pub fn dependency(&self, path: &MessagePath) -> Option<&Msg> {
        self.dependencies.get(path)
    }

    fn parse_msg(message_type: &str, message_src: &str) -> Result<Msg> {
        let message_path = message_type.try_into()?;
        let msg = Msg::new(message_path, message_src)?;
        Ok(msg)
    }

    fn parse_dependency(message_body: &str) -> Result<Msg> {
        lazy_static! {
            static ref RE_DESCRIPTOR_MSG_TYPE: regex::Regex =
                regex::Regex::new(r#"^\s*MSG:\s*(\S+)\s*$"#).unwrap();
        }
        let message_body = message_body.trim();
        let (message_type_line, message_src) =
            message_body
                .split_once('\n')
                .ok_or(Error::BadMessageContent(format!(
                    "Message dependency is missing type declaration"
                )))?;
        let cap =
            RE_DESCRIPTOR_MSG_TYPE
                .captures(message_type_line)
                .ok_or(Error::BadMessageContent(format!(
                    "Failed to parse message type line `{}`",
                    message_type_line
                )))?;
        let message_type = cap.get(1).ok_or(Error::BadMessageContent(format!(
            "Failed to parse message type line `{}`",
            message_type_line
        )))?;
        Self::parse_msg(message_type.as_str(), message_src)
    }

    fn get_dependency(&self, path: &MessagePath) -> Result<&Msg> {
        self.dependencies
            .get(path)
            .ok_or(Error::MessageDependencyMissing {
                package: path.package().to_owned(),
                name: path.name().to_owned(),
            })
    }

    /// This will read the provided reader to its end and return a map with all field names mapped to their values.
    /// If you encounter perfomance problems you may want to take a look at the  [`Self::decode_raw()`] method instead
    pub fn decode(&self, r: &mut impl io::Read) -> Result<MyMap<String, Value>> {
        let values = self.decode_message(self.msg(), r)?;
        let mut map = MyMap::default();
        for (i, v) in values.into_iter().enumerate() {
            map.insert(self.msg().fields()[i].name().to_owned(), v);
        }

        Ok(map)
    }

    /// This will read the provided reader to its end and return a vector with all field values.
    /// The values are in the same order as the message fields, this is done to save cost from expensive
    /// Map read&writes.
    ///
    /// If you want a HashMap of values instead use the [`Self::decode()`] method instead
    pub fn decode_raw<R: Read>(&self, r: R) -> Result<MessageValues> {
        self.decode_message(self.msg(), r)
    }

    // This is necessary to prevent the creation of nested ByteCounters
    fn decode_message<R: Read>(&self, msg: &Msg, r: R) -> Result<MessageValues> {
        let mut r = ByteCounter::new(r);

        let mut buf = [0, 0, 0, 0];
        r.read_exact(&mut buf)?;

        // https://github.com/foxglove/cdr/blob/main/src/EncapsulationKind.ts
        // let kind = buf[1];
        if buf != [0, 0x01, 0, 0] {
            return Err(io::Error::other(format!(
                "Invalid CRD kind {:b}, only little endian is supported",
                buf[1]
            ))
            .into());
        }

        let decoded_values = self.decode_message_inner(msg, &mut r)?;

        // This is purely a sanity check
        {
            // Read alignment bytes
            let _ = r.align_to(4);

            // Ensure we read the entire message
            let mut buf = Vec::new();
            r.read_to_end(&mut buf)?;

            if buf != [] as [u8; 0] {
                println!("{:?}", buf);

                println!("{}", msg);

                println!("{:#?}", decoded_values);

                return Err(io::Error::other(format!(
                            "Encountered error after reading message, most likely the message padding was read wrong, please report this issue"
                        )).into());
            }
        }

        Ok(decoded_values)
    }

    fn decode_message_inner<R: Read>(
        &self,
        msg: &Msg,
        r: &mut ByteCounter<R>,
    ) -> Result<MessageValues> {
        msg.fields()
            .iter()
            .map(|field| {
                match field.case() {
                    FieldCase::Const(val) => Ok(Value::String(val.clone())),
                    FieldCase::Default(_) => self.decode_field(msg.path(), field, r),
                    FieldCase::Unit => self.decode_field(msg.path(), field, r),
                    //.expect("Error while decoding unit field"),
                    FieldCase::Vector => self.decode_field_array(msg.path(), field, None, r),
                    //.expect("Error while decoding vector field"),
                    FieldCase::Array(l) => self.decode_field_array(msg.path(), field, Some(*l), r), //.expect("Error while decoding array field"),
                }
                .map_err(|e| match e {
                    Error::DecodingError {
                        msg: _,
                        field: _,
                        offset: _,
                        err,
                    } => Error::DecodingError {
                        msg: msg.clone(),
                        field: field.clone(),
                        offset: r.bytes_read(),
                        err,
                    },
                    e => e,
                })
            })
            .try_collect()
    }

    fn decode_field<R: Read>(
        &self,
        parent: &MessagePath,
        field: &FieldInfo,
        r: &mut ByteCounter<R>,
    ) -> Result<Value> {
        let value = match field.datatype() {
            DataType::Bool => r.read_u8().map(|i| i != 0)?.into(),
            DataType::I8(_) => r.read_i8()?.into(),
            DataType::I16 => {
                r.align_to(2)?;
                r.read_i16::<LE>()?.into()
            }
            DataType::I32 => {
                r.align_to(4)?;
                r.read_i32::<LE>()?.into()
            }
            DataType::I64 => {
                r.align_to(ALIGNMENT)?;
                r.read_i64::<LE>()?.into()
            }
            DataType::U8(_) => r.read_u8()?.into(),
            DataType::U16 => {
                r.align_to(2)?;
                r.read_u16::<LE>()?.into()
            }
            DataType::U32 => {
                r.align_to(4)?;
                r.read_u32::<LE>()?.into()
            }
            DataType::U64 => {
                r.align_to(ALIGNMENT)?;
                r.read_u64::<LE>()?.into()
            }
            DataType::F32 => {
                r.align_to(4)?;
                r.read_f32::<LE>()?.into()
            }
            DataType::F64 => {
                r.align_to(ALIGNMENT)?;
                r.read_f64::<LE>()?.into()
            }
            DataType::String => {
                r.align_to(4)?;
                let len = r.read_u32::<LE>()?;

                if len == 0 {
                    return Ok(Value::String("".to_owned()));
                }

                let mut v = vec![0; (len - 1) as usize];
                r.read_exact(&mut v)?;

                // Read \0 character
                let null = r.read_u8()?;
                assert_eq!(0, null);

                match String::from_utf8(v) {
                    Ok(s) => Value::String(s),
                    Err(e) => {
                        return Err(Error::DecodingError {
                            err: std::io::Error::other(e),
                            field: field.clone(),
                            msg: self.msg.clone(),
                            offset: r.bytes_read(),
                        })
                    }
                }
            }
            DataType::Time => {
                r.align_to(4)?;
                let sec = r.read_u32::<LE>()?;
                let nsec = r.read_u32::<LE>()?;

                return Ok(Value::Time(crate::Time { sec, nsec }));
            }
            DataType::Duration => panic!("Duration parsing not implemented yet"),
            DataType::LocalMessage(name) => {
                let path = parent.peer(name);
                let dependency = self.get_dependency(&path)?;

                // Decoding is fully unmapped so messages are just expressed as
                // arrays before they get mapped to field names
                Value::Array(self.decode_message_inner(dependency, r)?)
            }
            DataType::GlobalMessage(path) => {
                // panic!("Global messages unsupported (Hasher) {path}");

                let dependency = self.get_dependency(path)?;
                self.decode_message_inner(dependency, r)?.into()
            }
        };

        Ok(value)
    }

    fn decode_field_array<R: Read>(
        &self,
        parent: &MessagePath,
        field: &FieldInfo,
        array_length: Option<usize>,
        r: &mut ByteCounter<R>,
    ) -> Result<Value> {
        let array_length = match array_length {
            Some(v) => v,
            None => r.read_u32::<LE>()? as usize,
        };
        // TODO: optimize by checking data type only once
        println!("{}", array_length);
        (0..array_length)
            .map(|_| self.decode_field(parent, field, r))
            .collect()
    }
}

struct ByteCounter<R> {
    inner: R,
    count: usize,
}

impl<R> ByteCounter<R>
where
    R: Read,
{
    fn new(inner: R) -> Self {
        ByteCounter { inner, count: 0 }
    }

    /*
    fn into_inner(self) -> R {
        self.inner
    }
    */

    fn bytes_read(&self) -> usize {
        self.count
    }

    /// Read the necessary amount of bytes so that the next read will be aligned to `size` bytes
    fn align_to(&mut self, size: usize) -> io::Result<()> {
        let cur_pos = self.bytes_read();
        let cur_align = cur_pos % size;

        if cur_align > 0 {
            let needed_offset = size - cur_align;

            // println!("Aligning from {} + {}", cur_pos, needed_offset);

            let mut buf = vec![0; needed_offset];
            self.read_exact(&mut buf)?;
        }

        // println!("Next byte will be {}", self.bytes_read());

        Ok(())
    }
}

impl<R> Read for ByteCounter<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.inner.read(buf);
        if let Ok(size) = res {
            self.count += size
        }
        res
    }
}
