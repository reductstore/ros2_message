use crate::dynamic::DynamicMsg;
use crate::Value;
use std::hash::RandomState;
use std::io::Cursor;

#[test]
fn decoding_nested_message() {
    let msg_definition = r#"
builtin_interfaces/Time stamp
float32 value

================================================================================
MSG: builtin_interfaces/Time

int32 sec
uint32 nanosec
            "#;

    let dynamic_message: DynamicMsg<RandomState> =
        DynamicMsg::new("package/msg/SmallMsg", msg_definition)
            .expect("The message definition was invalid");
    let message = dynamic_message
        .decode(
            &[
                0x00u8, 0x01, 0, 0, 157, 47, 136, 102, 42, 0, 0, 0, 219, 15, 73, 64,
            ][..],
        )
        .expect("The supplied bytes do not match the message definition");

    // Reading the value field of the message
    assert_eq!(message["value"], Value::F32(core::f32::consts::PI));
    let stamp = message["stamp"].as_map().unwrap();
    assert_eq!(stamp["sec"], Value::I32(1720201117));
    assert_eq!(stamp["nanosec"], Value::U32(42));
}

/// Minimal repro for ros2_message padding bug when decoding sensor_msgs/msg/CameraInfo
/// using only raw schema and bytes.
#[test]
fn ros2_message_camera_info_decode_fails() {
    // Schema lifted directly from examples/test_bag_sqlite3 for sensor_msgs/msg/CameraInfo.
    const CAMERA_INFO_SCHEMA: &str = r#"std_msgs/Header header
uint32 height
uint32 width
string distortion_model
float64[] d
float64[9] k
float64[9] r
float64[12] p
uint32 binning_x
uint32 binning_y
sensor_msgs/RegionOfInterest roi
================================================================================
MSG: std_msgs/Header
builtin_interfaces/Time stamp
string frame_id
================================================================================
MSG: builtin_interfaces/Time
int32 sec
uint32 nanosec
================================================================================
MSG: sensor_msgs/RegionOfInterest
uint32 x_offset
uint32 y_offset
uint32 height
uint32 width
bool do_rectify
"#;

    // Raw CameraInfo sample that triggers padding error in ros2_message::DynamicMsg decoding.
    const CAMERA_INFO_BYTES: &[u8] = &[
        0, 1, 0, 0, 252, 110, 68, 104, 0, 33, 204, 42, 11, 0, 0, 0, 116, 101, 115, 116, 95, 102,
        114, 97, 109, 101, 0, 0, 224, 1, 0, 0, 128, 2, 0, 0, 10, 0, 0, 0, 112, 108, 117, 109, 98,
        95, 98, 111, 98, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 154, 153, 153, 153, 153, 153, 185, 63,
        154, 153, 153, 153, 153, 153, 201, 191, 252, 169, 241, 210, 77, 98, 80, 63, 252, 169, 241,
        210, 77, 98, 96, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 128, 64, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 116, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 128, 64, 0, 0,
        0, 0, 0, 0, 110, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240,
        63, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 104, 128, 64, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 116, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 104, 128, 64, 0, 0, 0, 0, 0, 0, 110, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let msg: DynamicMsg<RandomState> =
        DynamicMsg::new("sensor_msgs/CameraInfo", CAMERA_INFO_SCHEMA)
            .expect("failed to build DynamicMsg");

    msg.decode(&mut Cursor::new(CAMERA_INFO_BYTES)).unwrap();
}

/// Ensure decoding an empty schema does not panic or error.
#[test]
fn decode_empty_message() {
    let empty_schema = "";
    let msg: DynamicMsg<RandomState> =
        DynamicMsg::new("std_msgs/msg/Empty", empty_schema).expect("failed to build DynamicMsg");

    let decoded = msg
        .decode(&[0u8, 1, 0, 0][..])
        .expect("empty messages should decode without error");

    assert!(decoded.is_empty(), "Empty message should yield no fields");
}
