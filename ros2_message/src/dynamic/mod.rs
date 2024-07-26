#[cfg(feature = "decode")]
mod decode;
#[cfg(feature = "decode")]
pub use decode::DynamicMsg;

#[cfg(feature = "mcap")]
mod mcap;
#[cfg(feature = "mcap")]
pub use mcap::McapMessageStream;
#[cfg(feature = "mcap")]
pub use mcap::UnmappedMcapMessageStream;
