//! Crate for describing ROS messages, and handling MSG and SRV files.
//! <div class="warning">
//!
//! This crate is not extensively tested yet and there is a good chance that you will discover issues.
//! If you do please [create a GitHub Issue](https://github.com/A-K-O-R-A/ros2_message/issues/new)
//!
//! </div>
#![warn(missing_docs)]

mod data_type;

mod error;
mod field_info;
mod message_path;
mod msg;
mod parse_msg;
mod srv;
#[cfg(test)]
mod tests;
mod time;
mod value;

/// Provides functionality by decoding a ROS2 messages when the message definition is
/// only known at runtime, see [dynamic::DynamicMsg] for more.
#[cfg(feature = "decode")]
pub mod dynamic;

pub use data_type::{DataType, I8Variant, U8Variant};
pub use error::{Error, Result};
pub use field_info::{FieldCase, FieldInfo};
pub use message_path::MessagePath;
pub use msg::Msg;
pub use srv::Srv;
pub use time::{Duration, Time};
pub use value::{MessageValue, Value};
