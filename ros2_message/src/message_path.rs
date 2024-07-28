use crate::{Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::{BuildHasher, Hash, RandomState};

/// Path to a ROS message with naming conventions tested.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "&str")]
pub struct MessagePath {
    package: String,
    name: String,
}

pub fn is_valid_package_name(package: &str) -> bool {
    lazy_static! {
        static ref RE_PACKAGE_CORRECT_CHAR_SET_AND_LENGTH: Regex =
            Regex::new("^[a-z][a-z0-9_]+$").unwrap();
        static ref RE_PACKAGE_CONSECUTIVE_UNDERSCORE: Regex = Regex::new("__").unwrap();
    }
    RE_PACKAGE_CORRECT_CHAR_SET_AND_LENGTH.is_match(package)
        && !RE_PACKAGE_CONSECUTIVE_UNDERSCORE.is_match(package)
}

impl MessagePath {
    /// Create full message path, with naming rules checked
    ///
    /// Naming rules are based on [REP 144](https://www.ros.org/reps/rep-0144.html).
    ///
    /// # Errors
    ///
    /// An error will be returned if naming conventions are not met.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::MessagePath;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let message_path = MessagePath::new("foo", "Bar")?;
    ///
    /// assert_eq!(message_path.package(), "foo");
    /// assert_eq!(message_path.name(), "Bar");
    ///
    /// assert!(MessagePath::new("0foo", "Bar").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<S: BuildHasher + Default + Clone + core::fmt::Debug>(
        package: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, S> {
        let package = package.into();
        let name = name.into();
        if !is_valid_package_name(&package) {
            return Err(Error::InvalidMessagePath  {
                name: format!("{}/{}",package,name),
                  reason: "package name needs to follow REP 144 rules (https://www.ros.org/reps/rep-0144.html)".into(),
            });
        }
        Ok(Self { package, name })
    }

    /// Create a new message path inside the same package.
    ///
    /// Prevents the need for constant error checking of package names.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::MessagePath;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let message_path = MessagePath::new("foo", "Bar")?.peer("Baz");
    ///
    /// assert_eq!(message_path.package(), "foo");
    /// assert_eq!(message_path.name(), "Baz");
    /// # Ok(())
    /// # }
    /// ```
    pub fn peer(&self, name: impl Into<String>) -> Self {
        Self {
            package: self.package.clone(),
            name: name.into(),
        }
    }

    fn from_combined<S: BuildHasher + Default + Clone + core::fmt::Debug>(
        input: &str,
    ) -> Result<Self, S> {
        let parts = input.splitn(3, '/').collect::<Vec<&str>>();
        match parts[..] {
            [package, msg, name] => match msg {
                "srv" => Err(Error::InvalidMessagePath {
                    name: input.to_owned(),
                    reason:
                        "service names are not valid message paths, please use ServicePath(not yet implemented) instead"
                            .into(),
                }),
                "msg" => Self::new(package, name),
                _ => Err(Error::InvalidMessagePath {
                    name: input.to_owned(),
                    reason: "message path should follow the pattern packageName/msg/messageName"
                        .into(),
                }),
            },
            [package, name] => Self::new(package, name),
            _ => Err(Error::InvalidMessagePath {
                name: input.into(),
                reason: "string needs to be in `package/name` format".into(),
            }),
        }
    }

    /// Package that the message is located in.
    pub fn package(&self) -> &str {
        &self.package
    }

    /// Name of the message inside the package.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for MessagePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.package(), self.name())
    }
}

impl<'a> TryFrom<&'a str> for MessagePath {
    type Error = Error<RandomState>;

    fn try_from(value: &'a str) -> Result<Self, RandomState> {
        Self::from_combined(value)
    }
}

impl From<MessagePath> for String {
    fn from(src: MessagePath) -> Self {
        format!("{}", src)
    }
}
