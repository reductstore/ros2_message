use crate::{Duration, Time};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{BuildHasher, RandomState};
use std::iter::FromIterator;

use derive_where::derive_where;

/// Represents an arbitrary ROS message.
pub type MessageValue<S> = HashMap<String, Value<S>, S>; //  = RandomState

#[derive(PartialEq, Eq, Serialize)]
pub struct A<S: BuildHasher>(HashMap<String, String, S>); //  = RandomState

/// Represents an arbitrary ROS message or value in it.
#[derive(Serialize, Deserialize)]
#[derive_where(Clone, PartialEq, Debug)]
pub enum Value<S: BuildHasher + Default + Clone + core::fmt::Debug = RandomState> {
    //  = RandomState
    /// Represents `bool`.
    Bool(bool),
    /// Represents `int8` or `byte`.
    I8(i8),
    /// Represents `int16`.
    I16(i16),
    /// Represents `int32`.
    I32(i32),
    /// Represents `int64`.
    I64(i64),
    /// Represents `uint8` or `char`.
    U8(u8),
    /// Represents `uint16`.
    U16(u16),
    /// Represents `uint32`.
    U32(u32),
    /// Represents `uint64`.
    U64(u64),
    /// Represents `float32`.
    F32(f32),
    /// Represents `float64`.
    F64(f64),
    /// Represents `string`.
    String(String),
    /// Represents `time`.
    Time(Time),
    /// Represents `duration`.
    Duration(Duration),
    /// Represents `some_type[]` or `some_type[length]`.
    ///
    /// For example: `float32[64]`, `geometry_msgs/Point[]`.
    Array(Vec<Value<S>>),
    /// Represents an embedded message.
    Message(HashMap<String, Value<S>, S>),
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> Value<S> {
    fn fmt_indented(&self, indentation: usize, step: usize, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => v.fmt(f),
            Value::I8(v) => v.fmt(f),
            Value::I16(v) => v.fmt(f),
            Value::I32(v) => v.fmt(f),
            Value::I64(v) => v.fmt(f),
            Value::U8(v) => v.fmt(f),
            Value::U16(v) => v.fmt(f),
            Value::U32(v) => v.fmt(f),
            Value::U64(v) => v.fmt(f),
            Value::F32(v) => v.fmt(f),
            Value::F64(v) => v.fmt(f),
            Value::String(v) => write!(f, "{:?}", v),
            Value::Time(v) => v.fmt(f),
            Value::Duration(v) => v.fmt(f),
            Value::Array(items) => {
                for item in items {
                    writeln!(f)?;
                    write!(f, "{:indent$}- ", "", indent = indentation)?;
                    item.fmt_indented(indentation + step, step, f)?;
                }
                Ok(())
            }
            Value::Message(items) => {
                for (key, item) in items.iter().sorted_by(|a, b| Ord::cmp(&a.0, &b.0)) {
                    writeln!(f)?;
                    write!(f, "{:indent$}{}: ", "", key, indent = indentation)?;
                    item.fmt_indented(indentation + step, step, f)?;
                }
                Ok(())
            }
        }
    }

    /// Returns the content if `Value` is a `bool`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::Bool(true).as_bool(), Some(true));
    /// assert_eq!(Value::Bool(false).as_bool(), Some(false));
    /// assert!(Value::U32(12).as_bool().is_none());
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i8`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::I8(12).as_i8(), Some(12));
    /// assert!(Value::U32(12).as_i8().is_none());
    /// ```
    pub fn as_i8(&self) -> Option<i8> {
        if let Value::I8(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i16`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::I16(12).as_i16(), Some(12));
    /// assert!(Value::U32(12).as_i16().is_none());
    /// ```
    pub fn as_i16(&self) -> Option<i16> {
        if let Value::I16(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::I32(12).as_i32(), Some(12));
    /// assert!(Value::U32(12).as_i32().is_none());
    /// ```
    pub fn as_i32(&self) -> Option<i32> {
        if let Value::I32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::I64(12).as_i64(), Some(12));
    /// assert!(Value::U32(12).as_i64().is_none());
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        if let Value::I64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u8`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::U8(12).as_u8(), Some(12));
    /// assert!(Value::U32(12).as_u8().is_none());
    /// ```
    pub fn as_u8(&self) -> Option<u8> {
        if let Value::U8(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u16`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::U16(12).as_u16(), Some(12));
    /// assert!(Value::U32(12).as_u16().is_none());
    /// ```
    pub fn as_u16(&self) -> Option<u16> {
        if let Value::U16(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::U32(12).as_u32(), Some(12));
    /// assert!(Value::U16(12).as_u32().is_none());
    /// ```
    pub fn as_u32(&self) -> Option<u32> {
        if let Value::U32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::U64(12).as_u64(), Some(12));
    /// assert!(Value::U32(12).as_u64().is_none());
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        if let Value::U64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `f32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::F32(12.0).as_f32(), Some(12.0));
    /// assert!(Value::U32(12).as_f32().is_none());
    /// ```
    pub fn as_f32(&self) -> Option<f32> {
        if let Value::F32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::F64(12.0).as_f64(), Some(12.0));
    /// assert!(Value::U32(12).as_f64().is_none());
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        if let Value::F64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns a `&str` if `Value` is a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::String("foo".into()).as_str(), Some("foo"));
    /// assert!(Value::U32(12).as_str().is_none());
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(Value::String("foo".into()).try_into_string(), Some("foo".into()));
    /// assert!(Value::U32(12).try_into_string().is_none());
    /// ```
    pub fn try_into_string(self) -> Option<String> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `Time` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::{Time, Value};
    /// assert_eq!(
    ///     Value::Time(Time::from_nanos(120)).as_time(),
    ///     Some(Time::from_nanos(120)),
    /// );
    /// assert!(Value::U32(12).as_time().is_none());
    /// ```
    pub fn as_time(&self) -> Option<Time> {
        if let Value::Time(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `Duration` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::{Duration, Value};
    /// assert_eq!(
    ///     Value::Duration(Duration::from_nanos(120)).as_duration(),
    ///     Some(Duration::from_nanos(120)),
    /// );
    /// assert!(Value::U32(12).as_duration().is_none());
    /// ```
    pub fn as_duration(&self) -> Option<Duration> {
        if let Value::Duration(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns a reference to the content if `Value` is an array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(
    ///     Value::Array(vec![1u32.into(), 2u32.into(), 3u32.into()]).as_slice(),
    ///     Some(&[Value::U32(1), Value::U32(2), Value::U32(3)][..]),
    /// );
    /// assert!(Value::U32(12).as_slice().is_none());
    /// ```
    pub fn as_slice(&self) -> Option<&[Value<S>]> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// assert_eq!(
    ///     Value::Array(vec![1u32.into(), 2u32.into(), 3u32.into()]).try_into_vec(),
    ///     Some(vec![Value::U32(1), Value::U32(2), Value::U32(3)]),
    /// );
    /// assert!(Value::U32(12).try_into_vec().is_none());
    /// ```
    pub fn try_into_vec(self) -> Option<Vec<Value<S>>> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns a reference to the content if `Value` is a message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// # use std::collections::HashMap;
    /// let mut data = HashMap::<String, Value<S>>::new();
    /// data.insert("foo".into(), true.into());
    /// data.insert("bar".into(), false.into());
    /// assert_eq!(Value::Message(data.clone()).as_map(), Some(&data));
    /// assert!(Value::U32(12).as_map().is_none());
    /// ```
    pub fn as_map(&self) -> Option<&MessageValue<S>> {
        if let Value::Message(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros2_message::Value;
    /// # use std::collections::HashMap;
    /// let mut data = HashMap::<String, Value<S>>::new();
    /// data.insert("foo".into(), true.into());
    /// data.insert("bar".into(), false.into());
    /// assert_eq!(Value::Message(data.clone()).try_into_map(), Some(data));
    /// assert!(Value::U32(12).try_into_map().is_none());
    /// ```
    pub fn try_into_map(self) -> Option<MessageValue<S>> {
        if let Value::Message(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub(crate) fn to_random_state(self) -> Value<RandomState> {
        match self {
            Value::Bool(v) => Value::Bool(v),
            Value::I8(v) => Value::I8(v),
            Value::I16(v) => Value::I16(v),
            Value::I32(v) => Value::I32(v),
            Value::I64(v) => Value::I64(v),
            Value::U8(v) => Value::U8(v),
            Value::U16(v) => Value::U16(v),
            Value::U32(v) => Value::U32(v),
            Value::U64(v) => Value::U64(v),
            Value::F32(v) => Value::F32(v),
            Value::F64(v) => Value::F64(v),
            Value::String(v) => Value::String(v),
            Value::Time(v) => Value::Time(v),
            Value::Duration(v) => Value::Duration(v),
            Value::Array(v) => Value::Array(v.into_iter().map(Self::to_random_state).collect()),
            Value::Message(v) => Value::Message(
                v.into_iter()
                    .map(|(k, v)| (k, v.to_random_state()))
                    .collect(),
            ),
        }
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> Display for Value<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_indented(0, 2, f)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<bool> for Value<S> {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<i8> for Value<S> {
    fn from(v: i8) -> Self {
        Self::I8(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<i16> for Value<S> {
    fn from(v: i16) -> Self {
        Self::I16(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<i32> for Value<S> {
    fn from(v: i32) -> Self {
        Self::I32(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<i64> for Value<S> {
    fn from(v: i64) -> Self {
        Self::I64(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<u8> for Value<S> {
    fn from(v: u8) -> Self {
        Self::U8(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<u16> for Value<S> {
    fn from(v: u16) -> Self {
        Self::U16(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<u32> for Value<S> {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<u64> for Value<S> {
    fn from(v: u64) -> Self {
        Self::U64(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<f32> for Value<S> {
    fn from(v: f32) -> Self {
        Self::F32(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<f64> for Value<S> {
    fn from(v: f64) -> Self {
        Self::F64(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<String> for Value<S> {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<Time> for Value<S> {
    fn from(v: Time) -> Self {
        Self::Time(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<Duration> for Value<S> {
    fn from(v: Duration) -> Self {
        Self::Duration(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug, T: Into<Value<S>>> From<Vec<T>>
    for Value<S>
{
    fn from(v: Vec<T>) -> Self {
        Self::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug, T: Into<Value<S>>, const L: usize>
    From<[T; L]> for Value<S>
{
    fn from(v: [T; L]) -> Self {
        Self::Array(IntoIterator::into_iter(v).map(Into::into).collect())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> From<HashMap<String, Value<S>, S>>
    for Value<S>
{
    fn from(v: HashMap<String, Value<S>, S>) -> Self {
        Self::Message(v)
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for bool {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_bool().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for i8 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_i8().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for i16 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_i16().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for i32 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_i32().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for i64 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_i64().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for u8 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_u8().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for u16 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_u16().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for u32 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_u32().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for u64 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_u64().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for f32 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_f32().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for f64 {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_f64().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for String {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.try_into_string().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for Time {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_time().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>> for Duration {
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.as_duration().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug, T: TryFrom<Value<S>>> TryFrom<Value<S>>
    for Vec<T>
{
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        let value = value.try_into_vec().ok_or(())?;
        value
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Self, _>>()
            .map_err(|_| ())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug, T: TryFrom<Value<S>>, const L: usize>
    TryFrom<Value<S>> for [T; L]
{
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        let value = value.try_into_vec().ok_or(())?;
        if value.len() != L {
            return Err(());
        }
        array_init::from_iter(value.into_iter().filter_map(|v| v.try_into().ok())).ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug> TryFrom<Value<S>>
    for HashMap<String, Value<S>, S>
{
    type Error = ();

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        value.try_into_map().ok_or(())
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug, K: Into<String>, T: Into<Value<S>>>
    FromIterator<(K, T)> for Value<S>
{
    fn from_iter<I: IntoIterator<Item = (K, T)>>(iter: I) -> Self {
        Self::Message(
            iter.into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl<S: BuildHasher + Default + Clone + core::fmt::Debug, T: Into<Value<S>>> FromIterator<T>
    for Value<S>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Into::into).collect())
    }
}
