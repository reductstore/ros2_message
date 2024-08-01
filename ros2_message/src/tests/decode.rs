use std::hash::RandomState;

use crate::dynamic::DynamicMsg;
use crate::Value;

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
