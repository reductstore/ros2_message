# ros2_message

[![MIT Licensed](https://img.shields.io/crates/l/ros2_message.svg?maxAge=3600)](./LICENSE)

This repository is an active fork of [A-K-O-R-A/ros2_message](https://github.com/A-K-O-R-A/ros2_message) maintained by ReductStore. We actively use and maintain the `DynamicMsg` functionality for runtime decoding in our projects.

## Quick start: DynamicMsg

```rust
use ros2_message::dynamic::DynamicMsg;
use std::hash::RandomState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a message schema at runtime
    let schema = r#"
int32 data
string label
    "#;

    let msg = DynamicMsg::<RandomState>::new("example_msgs/Custom", schema)?;

    // Little-endian CDR bytes: data = 42, label = "hello"
    let bytes = [0, 0, 0, 0, 42, 0, 0, 0, 6, 0, 0, 0, b'h', b'e', b'l', b'l', b'o', 0];
    let decoded = msg.decode(&bytes[..])?;

    assert_eq!(decoded["data"].as_i32(), Some(42));
    assert_eq!(decoded["label"].as_str(), Some("hello"));
    Ok(())
}
```
