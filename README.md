# ros2_message

[![MIT Licensed](https://img.shields.io/crates/l/ros2_message.svg?maxAge=3600)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ros2_message.svg?maxAge=3600)](https://crates.io/crates/ros2_message)

> [!WARNING]
> This crate is not extensively tested yet and there is a good chance that you will discover issues.
> If you do please [create a GitHub Issue](https://github.com/A-K-O-R-A/ros2_message/issues/new)

**ros2_message** is a utility library for handling anything related to [ROS2](https://ros.org/) `.msg` and `.srv` files in Rust. It's primary focus is parsing and dynamically decoding ROS messages.


## Resources
 - ROS2 documentation https://docs.ros.org/
 - Interface documentation https://docs.ros.org/en/iron/Concepts/Basic/About-Interfaces.html#messages

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
