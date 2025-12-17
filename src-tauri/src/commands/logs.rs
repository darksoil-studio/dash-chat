use anyhow::anyhow;
use dashchat_node::{
    spaces::{SpaceControlMessage, SpacesArgs, TestConditions},
    topic::LogId,
    ChatId, Header, Node, Payload, Topic,
};
use futures::future::try_join_all;
use p2panda_core::{cbor::decode_cbor, Body, Hash, PublicKey};
use p2panda_net::TopicId;
use p2panda_spaces::{
    event::{GroupEvent, SpaceEvent},
    Event,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SimplifiedOperation {
    // hash: Hash,
    pub header: SimplifiedHeader,
    pub body: Option<serde_json::Value>,
}

// #[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
// pub struct SimplifiedSpacesOperation {
//     // hash: Hash,
//     pub header: SimplifiedHeader,
//     pub events: Vec<Event<ChatId, TestConditions>>,
// }

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SimplifiedHeader {
    /// Author of this operation.
    public_key: PublicKey,

    /// Time in microseconds since the Unix epoch.
    timestamp: u64,

    /// Number of operations this author has published to this log, begins with 0 and is always
    /// incremented by 1 with each new operation by the same author.
    seq_num: u64,

    /// Hash of the previous operation of the same author and log. Can be omitted if first
    /// operation in log.
    backlink: Option<Hash>,

    /// List of hashes of the operations we refer to as the "previous" ones. These are operations
    /// from other authors. Can be left empty if no partial ordering is required or no other
    /// author has been observed yet.
    previous: Vec<Hash>,

    topic_id: Topic,
}

impl From<Header> for SimplifiedHeader {
    fn from(header: Header) -> SimplifiedHeader {
        SimplifiedHeader {
            public_key: header.public_key,
            timestamp: header.timestamp,
            seq_num: header.seq_num,
            backlink: header.backlink,
            previous: header.previous,
            topic_id: Topic::untyped(header.extensions.log_id.id()),
        }
    }
}

fn spaces_messages(payload: Payload) -> Option<Vec<SpaceControlMessage>> {
    match payload {
        Payload::Chat(dashchat_node::ChatPayload::Space(space_messages)) => Some(space_messages),
        _ => None,
    }
}

// pub async fn simplify_spaces_operation(
//     node: &Node,
//     // hash: Hash,
//     header: Header,
//     body: Option<Body>,
// ) -> anyhow::Result<SimplifiedSpacesOperation> {
//     let events: Vec<Event<ChatId, TestConditions>> = match body {
//         Some(b) => {
//             let payload: Payload = decode_cbor(&b.to_bytes()[..])?;

//             let spaces_messages = spaces_messages(payload)?;
//             let mut all_events: Vec<Event<ChatId, TestConditions>> = vec![];

//             for message in spaces_messages {
//                 let mut events = node.manager.process(&message).await?;

//                 all_events.append(&mut events);
//             }
//             all_events
//         }
//         _ => vec![],
//     };

//     let operation = SimplifiedSpacesOperation {
//         // hash,
//         header: SimplifiedHeader::from(header),
//         events,
//     };

//     Ok(operation)
// }
// pub fn decode_body(body: Body) -> Result<serde_json::Value, String> {
//     let bytes = body.to_bytes();
//     let Ok(Payload::Space(args)) = decode_cbor(&bytes[..]) else {
//         return Ok(decode_cbor(&bytes[..]).map_err(|err| format!("{err:?}"))?);
//     };
//     if let Some(value) = decode_spaces_args(args)? {
//         values.push(value);
//     }

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SimplifiedEvent {
    Application(serde_json::Value),
    Group(GroupEvent<TestConditions>),
    Space(SpaceEvent<ChatId>),
}

pub fn simplify_event(
    event: &Event<ChatId, TestConditions>,
) -> anyhow::Result<Option<SimplifiedEvent>> {
    match event {
        Event::Application { data, .. } => {
            let value: serde_json::Value = decode_cbor(&data[..])?;
            Ok(Some(SimplifiedEvent::Application(value)))
        }
        Event::Group(g) => Ok(Some(SimplifiedEvent::Group(g.clone()))),
        Event::Space(s) => Ok(Some(SimplifiedEvent::Space(s.clone()))),
        _ => Ok(None),
    }
}

pub async fn simplify(
    node: &Node,
    // hash: Hash,
    header: Header,
    body: Option<Body>,
) -> anyhow::Result<SimplifiedOperation> {
    let body: Option<serde_json::Value> = match body {
        Some(b) => {
            let payload: Payload = decode_cbor(&b.to_bytes()[..])?;

            if let Payload::Chat(dashchat_node::ChatPayload::Space(spaces_messages)) = payload {
                let mut all_events: Vec<SimplifiedEvent> = vec![];

                for message in spaces_messages {
                    // let events = node.manager.process(&message).await?;
                    let map = node.nodestate.spaces_events.read().await;
                    let Some(events) = map.get(&message.hash) else {
                        continue;
                    };
                    let mut simplified_events = events
                        .into_iter()
                        .map(simplify_event)
                        .collect::<anyhow::Result<Vec<Option<SimplifiedEvent>>>>()?
                        .into_iter()
                        .filter_map(|e| e)
                        .collect();

                    all_events.append(&mut simplified_events);
                }

                Some(serde_json::to_value(all_events)?)
            } else {
                Some(serde_json::to_value(payload)?)
            }
        }
        _ => None,
    };

    let operation = SimplifiedOperation {
        // hash,
        header: SimplifiedHeader::from(header),
        body,
    };

    Ok(operation)
}

#[tauri::command]
pub async fn get_log(
    topic_id: Topic,
    author: PublicKey,
    node: State<'_, Node>,
) -> Result<Vec<SimplifiedOperation>, String> {
    let log = node
        .get_log(LogId::from(topic_id), author)
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;

    let simplified_log = try_join_all(
        log.into_iter()
            .map(|(header, body)| simplify(&node, header, body)),
    )
    .await
    .map_err(|err| format!("{err:?}"))?;

    Ok(simplified_log)
}

#[tauri::command]
pub async fn get_authors(
    topic_id: Topic,
    node: State<'_, Node>,
) -> Result<std::collections::HashSet<PublicKey>, String> {
    let authors = node
        .get_authors(LogId::from(topic_id))
        .await
        .map_err(|e| format!("Failed to get log: {e:?}"))?;
    Ok(authors)
}
