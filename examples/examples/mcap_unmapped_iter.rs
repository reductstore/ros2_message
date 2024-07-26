use ros2_message::dynamic::UnmappedMcapMessageStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).expect("Provide a path to an .mcap file");

    let data = std::fs::read(path)?;
    let (stream, schemas) = UnmappedMcapMessageStream::new(&data)?;

    for raw_message in stream {
        let (unmapped, raw) = raw_message?;
        let channel_id = raw.header.channel_id as usize;

        // Get this channels message defition
        let Some(ref dynamic_msg) = schemas[channel_id] else {
            continue; // Skip channels with unknown schematas
        };

        let decoded_msg = dynamic_msg.map_values(unmapped)?;
        println!("{} {:#?}", dynamic_msg.msg().path().name(), decoded_msg);
    }

    Ok(())
}
