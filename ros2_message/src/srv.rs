use crate::{Error, MessagePath, Msg, Result};
use derive_where::derive_where;
use lazy_static::lazy_static;
use regex::RegexBuilder;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;
use std::hash::BuildHasher;

/// A ROS service parsed from a `srv` file.
#[derive(Serialize, Deserialize)]
#[derive_where(Clone, PartialEq, Eq, Hash, Debug)]
#[serde(into = "SrvSerde")]
#[serde(try_from = "SrvSerde")]
pub struct Srv<S: BuildHasher + Default + Clone + core::fmt::Debug> {
    path: MessagePath,
    source: String,
    req: Msg<S>,
    res: Msg<S>,
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> fmt::Display for Srv<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> Srv<S> {
    /// Create a service from a passed in path and source.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error parsing the service source.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Srv;
    /// # use std::convert::TryInto;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = Srv::new(
    ///     "foo/Bar".try_into()?,
    ///     r#"# a comment that is ignored
    ///     Header header
    ///     uint32 a
    ///     byte[16] b
    ///     geometry_msgs/Point[] point
    ///     uint32 FOO=5
    ///     string SOME_TEXT=this is # some text, don't be fooled by the hash
    /// ---
    ///     uint32 a
    ///     geometry_msgs/Point[] point
    ///     uint32 FOO=6
    ///     "#,
    /// )?;
    ///
    /// assert_eq!(service.path(), &"foo/Bar".try_into()?);
    /// assert_eq!(service.request().fields().len(), 6);
    /// assert_eq!(service.response().fields().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: MessagePath, source: impl Into<String>) -> Result<Srv<S>, S> {
        let source = source.into();
        let (req, res) = Self::build_req_res(&path, &source)?;
        Ok(Srv {
            path,
            source,
            req,
            res,
        })
    }

    /// Returns the path of the service.
    pub fn path(&self) -> &MessagePath {
        &self.path
    }

    /// Returns the original source.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the request message.
    pub fn request(&self) -> &Msg<S> {
        &self.req
    }

    /// Returns the response message.
    pub fn response(&self) -> &Msg<S> {
        &self.res
    }

    fn build_req_res(path: &MessagePath, source: &str) -> Result<(Msg<S>, Msg<S>), S> {
        lazy_static! {
            static ref RE_SPLIT: regex::Regex = RegexBuilder::new("^---$")
                .multi_line(true)
                .build()
                .expect("Invalid regex `^---$`");
        }
        let (req, res) = match RE_SPLIT.split(source).collect::<Vec<_>>().as_slice() {
            &[req] => (req, ""),
            &[req, res] => (req, res),
            &[] => {
                return Err(Error::BadMessageContent(format!(
                    "Service {} does not have any content",
                    path
                )))
            }
            v => {
                return Err(Error::BadMessageContent(format!(
                    "Service {} is split into {} parts",
                    path,
                    v.len()
                )))
            }
        };

        Ok((
            Msg::new(path.peer(format!("{}Req", path.name())), req)?,
            Msg::new(path.peer(format!("{}Res", path.name())), res)?,
        ))
    }
}

#[derive(Serialize, Deserialize)]
struct SrvSerde {
    path: MessagePath,
    source: String,
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<SrvSerde> for Srv<S> {
    type Error = Error<S>;

    fn try_from(src: SrvSerde) -> Result<Self, S> {
        Self::new(src.path, &src.source)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<Srv<S>> for SrvSerde {
    fn from(src: Srv<S>) -> Self {
        Self {
            path: src.path,
            source: src.source,
        }
    }
}
