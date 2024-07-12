use mcap::Summary;
use ros2_message::dynamic::DynamicMsg;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_summary(data: &[u8]) -> Result<Vec<Option<DynamicMsg>>> {
    let summary = Summary::read(data)?.expect("MCAP files without summary are not supported");

    let (&max_channel_id, _) = summary.channels.iter().max_by_key(|(id, _)| *id).unwrap();
    let max_channel_count = max_channel_id as usize + 1;

    // Message definition for a single channel accesible through:
    // message_definitions[channel_id]
    let mut message_definitions = vec![None; max_channel_count];

    println!(
        "{:<14} File contains {} channels with {} messages",
        "",
        summary.channels.len(),
        summary
            .stats
            .clone()
            .map_or("unknwon".to_owned(), |v| format!("{}", v.message_count))
    );

    // Parse message definitions for all channels
    for (id, channel) in &summary.channels {
        let Some(schema) = &channel.schema else {
            eprintln!("No schema found for channel {}", channel.topic);
            continue;
        };

        if schema.encoding != "ros2msg" {
            continue; // Ignore channels without ROS2 message
        }

        let msg_name = schema.name.clone();
        let msg_definition_string = String::from_utf8(schema.data.to_vec())?;
        let dynamic_msg = DynamicMsg::new(&msg_name, &msg_definition_string)?;

        // Save message definition
        message_definitions[*id as usize] = Some(dynamic_msg);
    }

    Ok(message_definitions)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).expect("Provide a path to an .mcap file");

    let data = std::fs::read(path)?;

    let message_definitions = parse_summary(&data)?;

    for raw_message in mcap::read::RawMessageStream::new(&data)? {
        let raw_message = raw_message?;
        let channel_id = raw_message.header.channel_id as usize;

        // Get this channels message defition
        let Some(ref dynamic_msg) = message_definitions[channel_id] else {
            continue; // Skip channels with unknown schematas
        };

        let decoded_msg = dynamic_msg.decode(raw_message.data.as_ref())?;
        println!("{} {:#?}", dynamic_msg.msg().path().name(), decoded_msg);
    }

    Ok(())
}
