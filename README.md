# ros2_message

[![MIT Licensed](https://img.shields.io/crates/l/ros2_message.svg?maxAge=3600)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ros2_message.svg?maxAge=3600)](https://crates.io/crates/ros2_message)

> [!WARNING]
> This crate is not extensively tested yet and there is a good chance that you will discover issues.
> If you do please [create a GitHub Issue](https://github.com/A-K-O-R-A/ros2_message/issues/new)

**ros2_message** is a utility library for handling anything related to [ROS2](https://ros.org/) `.msg` and `.srv` files in Rust.
It's primary focus is dynamically parsing and decoding ROS2 messages. The most common usecase for this is the [mcap](https://crates.io/crates/mcap) file format.
For more information take a look at the [`Msg`](https://docs.rs/ros2_message/latest/ros2_message/struct.Msg.html) and
[`DynamicMsg`](https://docs.rs/ros2_message/latest/ros2_message/dynamic/decode/struct.DynamicMsg.html) structs.

## Implementation status
 - [ ] Implementing the full CDR spec
   - [x] Little endian plain CDR is supported
   - [ ] Big endian plain CDR
   - [ ] PL CDR & CDR2
 - [ ] Extensive testing for different encoded messages


## Resources
 - ROS2 documentation https://docs.ros.org/
 - Interface documentation https://docs.ros.org/en/iron/Concepts/Basic/About-Interfaces.html#messages
 - All CDR types are specified under "7.4.3.4 Functions related to data types and objects" at https://www.omg.org/spec/DDS-XTypes/1.3/PDF

### Foxglove `.msg` file parsing implemenation
 - [foxglove/message-definition](https://github.com/foxglove/message-definition/)

### Foxglove dynamic decoding implementation
  - ROS2 specific deserialization:
    - [foxglove/rosmsg2-serialization/src/MessageReader.ts](https://github.com/foxglove/rosmsg2-serialization/blob/d262e58a47138c4725e0d7a4881eb45c7eea1cc2/src/MessageReader.ts)
  - Foxglove CDR implementation
    - [foxglove/cdr/src/CdrReader.ts](https://github.com/foxglove/cdr/blob/5cdd02e0be6fe5e7f4424f91d59d451f56459d33/src/CdrReader.ts)
  - Overview of all the CDR encapsulation kinds
    - [foxglove/cdr/src/EncapsulationKind.ts](https://github.com/foxglove/cdr/blob/5cdd02e0be6fe5e7f4424f91d59d451f56459d33/src/EncapsulationKind.ts)


    https://github.com/foxglove/rosmsg/tree/main
