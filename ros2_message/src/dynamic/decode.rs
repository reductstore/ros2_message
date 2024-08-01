use crate::error::{Error, Result};
use crate::{DataType, FieldCase, FieldInfo, MessagePath, Msg, Value};
use byteorder::{ReadBytesExt, LE};
use lazy_static::lazy_static;
use regex::RegexBuilder;
// use rustc_hash::FxHashMap;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::hash::{BuildHasher, RandomState};
use std::io::{self, Read};

pub(crate) type MessageValues<S> = VecDeque<Value<S>>;

// Most of this code is copied from
// https://github.com/adnanademovic/rosrust/blob/master/rosrust/src/dynamic_msg.rs

/// A dynamic Message provides a decoder for ROS2 messages at runtime without
/// the need to compile a message decoder during compilation. See [Self::new()] for more.
#[derive(Clone, Debug)]
pub struct DynamicMsg<S: BuildHasher + Default + Clone + core::fmt::Debug = RandomState> {
    // = RandomState> {
    msg: Msg<S>,
    dependencies: HashMap<MessagePath, Msg<S>, S>,
}

/// Byte alignment for CDR version 1
const ALIGNMENT: usize = 4;

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> DynamicMsg<S> {
    /// Create a new DynamicMsg<S> by parsing it's message definition. For this to work all of
    /// the messages depencies have to be provided as well, see the example for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use ros2_message::dynamic::DynamicMsg;
    ///
    /// let msg_definition = r#"
    /// builtin_interfaces/Time stamp
    /// float32 value
    ///
    /// ================================================================================
    /// MSG: builtin_interfaces/Time
    ///
    /// int32 sec
    /// uint32 nanosec
    /// "#;
    /// let dynamic_message = DynamicMsg::<std::hash::RandomState>::new("package/msg/SmallMsg<S>", msg_definition);
    /// assert!(dynamic_message.is_ok());
    /// ```
    pub fn new(message_name: &str, message_definition: &str) -> Result<Self> {
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
                message_name
            )))?;
        let msg = Self::parse_msg(message_name, message_src)?;
        let mut dependencies = HashMap::default();
        for message_body in message_bodies {
            let dependency = Self::parse_dependency(message_body)?;
            dependencies.insert(dependency.path().clone(), dependency);
        }

        Ok(DynamicMsg { msg, dependencies })
    }

    /// Returns the underlying ROS2 message definition
    pub fn msg(&self) -> &Msg<S> {
        &self.msg
    }

    /// Returns the associated dependency of the underlying parsed ROS2 message definition if present
    pub fn dependency(&self, path: &MessagePath) -> Option<&Msg<S>> {
        self.dependencies.get(path)
    }

    fn parse_msg(message_path: &str, message_src: &str) -> Result<Msg<S>> {
        let message_path = message_path.try_into()?;
        let msg = Msg::new(message_path, message_src)?;
        Ok(msg)
    }

    fn parse_dependency(message_body: &str) -> Result<Msg<S>> {
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

    fn get_dependency(&self, path: &MessagePath) -> Result<&Msg<S>> {
        let Some(msg) = self.dependencies.get(path) else {
            return Err(Error::MessageDependencyMissing {
                package: path.package().to_owned(),
                name: path.name().to_owned(),
            });
        };

        Ok(msg)
    }

    /// This will read the provided reader to its end and return a map with all field names mapped to their values.
    /// If you encounter perfomance problems you may want to take a look at the  [Self::decode_unmapped()] method instead
    ///
    /// # Errors
    ///
    /// This will error if the provided reader is either to long or too short, make sure you only porvide a reader for
    /// the byte slice containing exactly one message.
    ///
    /// # Examples
    ///
    /// ```
    /// use ros2_message::dynamic::DynamicMsg;
    ///
    /// let msg_definition = r#"
    /// builtin_interfaces/Time stamp
    /// float32 value
    ///
    /// ================================================================================
    /// MSG: builtin_interfaces/Time
    ///
    /// int32 sec
    /// uint32 nanosec
    /// "#;
    ///
    /// let dynamic_message = DynamicMsg::<std::hash::RandomState>::new("package/msg/SmallMsg<S>", msg_definition)
    ///     .expect("The message definition was invalid");
    /// let message = dynamic_message.decode(&[0x00u8, 0x01, 0, 0, 157, 47, 136, 102, 42, 0, 0 ,0, 219, 15, 73, 64][..])
    ///     .expect("The supplied bytes do not match the message definition");
    ///
    /// // Reading the value field of the message
    /// assert_eq!(message["value"], ros2_message::Value::F32(core::f32::consts::PI));
    /// ```
    pub fn decode<R: Read>(&self, r: R) -> Result<HashMap<String, Value<S>, S>> {
        let values = self.decode_message(self.msg(), r)?;

        self.map_values(values)
    }

    /// This maps the result of [Self::decode_unmapped()] to the result of [Self::decode()]
    pub fn map_values(&self, mut values: MessageValues<S>) -> Result<HashMap<String, Value<S>, S>> {
        self.map_field_names(self.msg(), &mut values)
    }

    // Map decoded field arrays to their field names for easy usage
    // TODO!: Use better collection method for this operation
    fn map_field_names(
        &self,
        msg: &Msg<S>,
        values: &mut MessageValues<S>,
    ) -> Result<HashMap<String, Value<S>, S>> {
        let mut map = HashMap::with_capacity_and_hasher(msg.fields().len(), Default::default());
        for field_info in msg.fields().iter() {
            let field_name = field_info.name().to_owned();

            // println!("{} - {:?}", &field_name, values);
            // !TODO: Error handling
            let value = values.pop_front().expect("what");

            let field_value = match field_info.datatype() {
                DataType::GlobalMessage(path) => {
                    let msg = self.get_dependency(&path)?;
                    let Value::Array(nested_values) = value else {
                        return Err(Error::DecodingError {
                                err: std::io::Error::other("Decoded message does not match the structure in the definition, please report this issue"),
                                field: field_info.clone().to_random_state(),
                                msg: msg.clone().to_random_state(),
                                offset: 0,
                            });
                    };

                    Value::Message(self.map_field_names(msg, &mut VecDeque::from(nested_values))?)
                }
                DataType::LocalMessage(name) => {
                    let msg = self.get_dependency(&msg.path().peer(name))?;
                    let Value::Array(nested_values) = value else {
                        return Err(Error::DecodingError {
                                err: std::io::Error::other("Decoded message does not match the structure in the definition, please report this issue"),
                                field: field_info.clone().to_random_state(),
                                msg: msg.clone().to_random_state(),
                                offset: 0,
                            });
                    };

                    Value::Message(self.map_field_names(msg, &mut VecDeque::from(nested_values))?)
                }
                _ => value,
            };

            map.insert(field_name, field_value);
        }

        Ok(map)
    }

    /// This will read the provided reader to its end and return a vector with all field values.
    /// The values are in the same order as the message fields, this is done to save cost from expensive
    /// Map read&writes.
    ///
    /// If you want a HashMap of values instead use the [Self::decode()] method instead
    ///
    /// # Examples
    ///
    /// ```
    /// use ros2_message::dynamic::DynamicMsg;
    ///
    /// let msg_definition = r#"
    /// builtin_interfaces/Time stamp
    /// float32 value
    ///
    /// ================================================================================
    /// MSG: builtin_interfaces/Time
    ///
    /// int32 sec
    /// uint32 nanosec
    /// "#;
    ///
    /// let dynamic_message = DynamicMsg::<std::hash::RandomState>::new("package/msg/SmallMsg<S>", msg_definition).expect("The message definition was invalid");
    /// let message = dynamic_message.decode_unmapped(&[0x00u8, 0x01, 0, 0, 157, 47, 136, 102, 42, 0, 0 ,0, 219, 15, 73, 64][..])
    ///     .expect("The supplied bytes do not match the message definition");
    ///
    /// // Reading the second field of the message
    /// assert_eq!(message[1], ros2_message::Value::F32(core::f32::consts::PI));
    /// ```
    pub fn decode_unmapped<R: Read>(&self, r: R) -> Result<MessageValues<S>> {
        self.decode_message(self.msg(), r)
    }

    // This is necessary to prevent the creation of nested ByteCounters
    fn decode_message<R: Read>(&self, msg: &Msg<S>, r: R) -> Result<MessageValues<S>> {
        let mut r = ByteCounter::new(r);

        let mut buf = [0, 0, 0, 0];
        r.read_exact(&mut buf)?;

        // https://github.com/foxglove/cdr/blob/main/src/EncapsulationKind.ts
        // let kind = buf[1];
        if buf != [0, 0x01, 0, 0] {
            return Err(Error::DecodingError {
                msg: msg.clone().to_random_state(),
                field: FieldInfo::new("uint8", "error_placeholder_field", crate::FieldCase::Unit)
                    .unwrap(),
                offset: r.bytes_read(),
                err: io::Error::other(format!(
                    "Invalid CRD kind {:b}, only little endian is supported",
                    buf[1]
                )),
            });
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
                return Err(io::Error::other(format!(
                            "Encountered error after reading message, most likely the message padding was read wrong,\
                             please report this issue. The message was decoded to the following fields:\n\n{decoded_values:#?}\n\n\
                             For further diagnosis please provide the following message definition:\n\n\
                             {msg}\n\nAlso provide the raw byte data: {buf:?}"
                        )).into());
            }
        }

        Ok(decoded_values)
    }

    fn decode_message_inner<R: Read>(
        &self,
        msg: &Msg<S>,
        r: &mut ByteCounter<R>,
    ) -> Result<MessageValues<S>> {
        let mut values = MessageValues::with_capacity(msg.fields().len());
        for field in msg.fields() {
            let res = match field.case() {
                FieldCase::Const(_) => Ok(field.const_value().unwrap().clone()),
                FieldCase::Unit | FieldCase::Default(_) => self.decode_field(msg.path(), field, r),
                //.expect("Error while decoding unit field"),
                FieldCase::Vector => self.decode_field_array(msg.path(), field, None, r),
                //.expect("Error while decoding vector field"),
                FieldCase::Array(l) => self.decode_field_array(msg.path(), field, Some(*l), r), //.expect("Error while decoding array field"),
            };

            let val = match res {
                Ok(v) => v,
                Err(e) => {
                    return Err(match e {
                        Error::DecodingError { err, .. } => Error::DecodingError {
                            msg: msg.clone().to_random_state(),
                            field: field.clone().to_random_state(),
                            offset: r.bytes_read(),
                            err,
                        },
                        e => e,
                    })
                }
            };

            values.push_back(val);
        }

        Ok(values)
    }

    fn decode_field<R: Read>(
        &self,
        parent: &MessagePath,
        field: &FieldInfo<S>,
        r: &mut ByteCounter<R>,
    ) -> Result<Value<S>> {
        /*
        let field_type = field.datatype().to_string();
        let prev_read = r.bytes_read();
        println!(" {} > at {:X}", field_type, prev_read);
        */

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
                            field: field.clone().to_random_state(),
                            msg: self.msg.clone().to_random_state(),
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
                Value::Array(self.decode_message_inner(dependency, r)?.into())
            }
            DataType::GlobalMessage(path) => {
                // panic!("Global messages unsupported (Hasher) {path}");

                let dependency = self.get_dependency(path)?;
                let vec: Vec<_> = self.decode_message_inner(dependency, r)?.into();

                vec.into()
            }
        };

        /*
        println!(
            " {} < at {:X} ( +{}) => {}",
            " ".repeat(field_type.len()),
            r.bytes_read(),
            r.bytes_read() - prev_read,
            &value
        );
         */

        Ok(value)
    }

    fn decode_field_array<R: Read>(
        &self,
        parent: &MessagePath,
        field: &FieldInfo<S>,
        array_length: Option<usize>,
        r: &mut ByteCounter<R>,
    ) -> Result<Value<S>> {
        let array_length = match array_length {
            Some(v) => v,
            None => r.read_u32::<LE>()? as usize,
        };
        // TODO: optimize by checking data type only once

        let mut values = Vec::with_capacity(array_length);
        for _ in 0..array_length {
            values.push(self.decode_field(parent, field, r)?);
        }

        Ok(Value::Array(values))
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
