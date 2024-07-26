use ros2_message::dynamic::McapMessageStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).expect("Provide a path to an .mcap file");

    let data = std::fs::read(path)?;

    for message in McapMessageStream::new(&data)? {
        let (msg, raw) = message?;
        let channel_id = raw.header.channel_id as usize;

        println!("Channel {}: {:#?}", channel_id, msg);
    }

    Ok(())
}
