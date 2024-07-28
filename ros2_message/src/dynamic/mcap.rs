use std::{
    hash::{BuildHasher, RandomState},
    ops::Deref,
};

use mcap::{
    read::{RawMessage, RawMessageStream},
    McapError, McapResult, Summary,
};

use super::DynamicMsg;

pub struct UnmappedMcapMessageStream<
    'a,
    S: BuildHasher + Default + Clone + core::fmt::Debug = RandomState,
> {
    message_definitions: Vec<Option<DynamicMsg<S>>>,
    raw_message_stream: mcap::read::RawMessageStream<'a>,
}

impl<'a, S: BuildHasher + Default + Clone + core::fmt::Debug> UnmappedMcapMessageStream<'a, S> {
    pub fn new<D: Deref<Target = [u8]>>(
        data: &'a D,
    ) -> McapResult<(Self, Vec<Option<DynamicMsg<S>>>)> {
        let Some(Summary { channels, .. }) = Summary::read(data)? else {
            // !TODO: proper error
            return Err(McapError::UnknownSchema("".into(), 0));
        };

        let max_channel_id = channels.iter().map(|(id, _)| id).max().unwrap_or(&0);
        let mut message_definitions = vec![None; (max_channel_id + 1) as usize];

        for (&id, channel) in &channels {
            let Some(schema) = &channel.schema else {
                continue;
            };

            if schema.encoding != "ros2msg" {
                continue;
            }

            let msg_name = schema.name.clone();
            // !TODO: Error handling
            let str_def = String::from_utf8(schema.data.to_vec()).unwrap();
            let dyn_msg = DynamicMsg::new(&msg_name, &str_def).unwrap();

            // Store message definition
            message_definitions[id as usize] = Some(dyn_msg);
        }

        let raw_message_stream = RawMessageStream::new(data)?;

        Ok((
            Self {
                message_definitions: message_definitions.clone(),
                raw_message_stream,
            },
            message_definitions,
        ))
    }
}

impl<'a, S: BuildHasher + Default + Clone + core::fmt::Debug> Iterator
    for UnmappedMcapMessageStream<'a, S>
{
    type Item = McapResult<(super::decode::MessageValues<S>, RawMessage<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_message = match self.raw_message_stream.next()? {
            Ok(m) => m,
            Err(e) => return Some(Err(e)),
        };

        let Some(ref dyn_msg) = self.message_definitions[raw_message.header.channel_id as usize]
        else {
            return None;
        };
        // !TODO: Error handling
        let decoded_msg = dyn_msg.decode_unmapped(&raw_message.data[..]).ok()?;

        Some(Ok((decoded_msg, raw_message)))
    }
}

pub struct McapMessageStream<'a, S: BuildHasher + Default + Clone + core::fmt::Debug = RandomState>
{
    message_definitions: Vec<Option<DynamicMsg<S>>>,
    unmapped_stream: UnmappedMcapMessageStream<'a, S>,
}

impl<'a, S: BuildHasher + Default + Clone + core::fmt::Debug> McapMessageStream<'a, S> {
    pub fn new<D: Deref<Target = [u8]>>(data: &'a D) -> McapResult<Self> {
        let (inner_stream, definitions) = UnmappedMcapMessageStream::new(data)?;

        Ok(Self {
            message_definitions: definitions,
            unmapped_stream: inner_stream,
        })
    }
}

impl<'a, S: BuildHasher + Default + Clone + core::fmt::Debug> Iterator
    for McapMessageStream<'a, S>
{
    type Item = McapResult<(crate::MessageValue<S>, RawMessage<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let (unmapped_msg, raw_message) = match self.unmapped_stream.next()? {
            Ok(m) => m,
            Err(e) => return Some(Err(e)),
        };

        let Some(ref dyn_msg) = self.message_definitions[raw_message.header.channel_id as usize]
        else {
            return None;
        };
        // !TODO: Error handling
        let decoded_msg = dyn_msg.map_values(unmapped_msg).ok()?;
        Some(Ok((decoded_msg, raw_message)))
    }
}
