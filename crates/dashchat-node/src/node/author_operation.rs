use p2panda_core::{Hash, Operation};
use p2panda_spaces::{OperationId, traits::AuthoredMessage};
use p2panda_stream::operation::IngestResult;

use crate::topic::{LogId, TopicKind};
use crate::{AsBody, testing::AliasedId};
use crate::{polestar as p, timestamp_now};

use super::*;

pub type OpStore = p2panda_store::MemoryStore<LogId, Extensions>;

impl Node {
    #[tracing::instrument(skip_all)]
    pub(super) async fn author_operation<K: TopicKind>(
        &self,
        topic: Topic<K>,
        payload: Payload,
        alias: Option<&str>,
    ) -> Result<Header<Extensions>, anyhow::Error> {
        self.author_operation_with_deps(topic, payload, vec![], alias)
            .await
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn author_operation_with_deps<K: TopicKind>(
        &self,
        topic: Topic<K>,
        payload: Payload,
        mut deps: Vec<p2panda_core::Hash>,
        alias: Option<&str>,
    ) -> Result<Header<Extensions>, anyhow::Error> {
        let mut sd = self.space_dependencies.write().await;

        let (ids, space_deps): (Vec<OperationId>, Vec<Hash>) = match &payload {
            Payload::Chat(p) => {
                match p {
                    ChatPayload::Space(msgs) => {
                        let ids = msgs.iter().map(|msg| msg.id()).collect::<Vec<_>>();
                        let op_deps = msgs.iter().flat_map(|msg| {
                            tracing::info!(msg = msg.id().alias(), "SM: authoring space msg");
                            msg.dependencies()
                        });

                        let mut header_deps = vec![];
                        for dep in op_deps {
                            // If the msg is part of the set being committed, it's ok
                            if ids.contains(&dep) {
                                continue;
                            }

                            match sd.get(&dep) {
                                Some(hash) => {
                                    header_deps.push(hash.clone());
                                    break;
                                }
                                None => {
                                    // XXX: This must not be allowed to happen, but for now
                                    // let's hope for the best!
                                    tracing::error!(
                                        dep = dep.alias(),
                                        deps = ?sd.keys().map(|k| k.alias()).collect::<Vec<_>>(),
                                        ids = ?ids.iter().map(|id| id.alias()).collect::<Vec<_>>(),
                                        "space dep should have been seen already",
                                    );
                                }
                            }
                        }

                        (ids, header_deps)
                    }

                    ChatPayload::JoinGroup(_chat_id) => (vec![], vec![]),
                }
            }
            Payload::Announcements(_) => (vec![], vec![]),
            Payload::Inbox(_) => (vec![], vec![]),
            Payload::DeviceGroup(_) => (vec![], vec![]),
        };

        deps.extend(space_deps.into_iter());

        let operation = create_operation(
            &self.op_store,
            &self.local_data.private_key,
            topic.clone(),
            payload.clone(),
            deps.clone(),
        )
        .await?;

        let Operation { header, body, hash } = operation;
        let log_id = header.extensions.log_id;

        if let Some(alias) = alias {
            header.hash().aliased(alias);
        }

        {
            let space_msgs = match &payload {
                Payload::Chat(ChatPayload::Space(msgs)) => {
                    msgs.iter().map(|m| m.id().alias()).collect::<Vec<_>>()
                }
                Payload::Chat(ChatPayload::JoinGroup(_)) => vec![],
                Payload::Inbox(_) => vec![],
                Payload::Announcements(_) => vec![],
                Payload::DeviceGroup(_) => vec![],
            };
            let pk = PK::from(header.public_key);
            tracing::info!(
                ?space_msgs,
                ?pk,
                hash = header.hash().alias(),
                deps = ?deps.iter().map(|id| id.alias()).collect::<Vec<_>>(),
                "authored operation"
            );
        }

        for id in ids {
            sd.insert(id, hash);
        }

        drop(sd);

        let result = p2panda_stream::operation::ingest_operation(
            &mut *self.op_store.clone(),
            header.clone(),
            body.clone(),
            header.to_bytes(),
            &topic.into(),
            false,
        )
        .await?;

        match result {
            IngestResult::Complete(op @ Operation { hash: hash2, .. }) => {
                assert_eq!(hash, hash2);

                // NOTE: if we fail to process here, incoming operations will be stuck as pending!
                self.process_ordering(op.clone()).await?;

                self.process_operation(op, self.author_store.clone(), true)
                    .await?;

                // self.notify_payload(&header, &payload).await?;
                tracing::debug!(?log_id, hash = hash.alias(), "authored operation");

                p::emit(p::Action::AuthorOp {
                    log_id,
                    hash: hash.clone(),
                });
            }

            IngestResult::Retry(h, _, _, missing) => {
                let backlink = h.backlink.as_ref().map(|h| h.alias());
                tracing::warn!(
                    ?topic,
                    hash = hash.alias(),
                    ?backlink,
                    ?missing,
                    "operation could not be ingested"
                );
            } // IngestResult::Duplicate(op) => {
              //     tracing::warn!(?topic, hash = hash.alias(), "operation already exists");
              //     return Ok(op.header);
              // }
        }

        match self.initialized_topics.read().await.get(&header.extensions.log_id) {
            Some(gossip) => {
                gossip
                    .send(ToNetwork::Message {
                        bytes: encode_gossip_message(&header, body.as_ref())?,
                    })
                    .await?;
            }
            None => {
                tracing::error!(?topic, "no gossip channel found for topic");
            }
        }

        Ok(header)
    }
}

pub(crate) async fn create_operation<K: TopicKind>(
    store: &OpStore,
    private_key: &PrivateKey,
    topic: Topic<K>,
    payload: Payload,
    deps: Vec<p2panda_core::Hash>,
) -> Result<Operation<Extensions>, anyhow::Error> {
    let public_key = private_key.public_key();
    let log_id = topic.clone();

    let body = Some(payload.try_into_body()?);

    let extensions = Extensions {
        log_id: log_id.clone().into(),
    };

    // TODO: atomicity, see https://github.com/p2panda/p2panda/issues/798
    let latest_operation = store.latest_operation(&public_key, &log_id.into()).await?;

    let (seq_num, backlink) = match latest_operation {
        Some((header, _)) => (header.seq_num + 1, Some(header.hash())),
        None => (0, None),
    };

    let timestamp = timestamp_now();

    let mut header = Header {
        version: 1,
        public_key,
        signature: None,
        payload_size: body.as_ref().map_or(0, |body| body.size()),
        payload_hash: body.as_ref().map(|body| body.hash()),
        timestamp,
        seq_num,
        backlink,
        previous: deps,
        extensions,
    };

    header.sign(private_key);

    Ok(Operation {
        hash: header.hash(),
        header,
        body,
    })
}
