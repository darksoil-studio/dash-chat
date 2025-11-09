use p2panda_core::{Hash, Operation};
use p2panda_net::TopicId;
use p2panda_spaces::{OperationId, traits::AuthoredMessage};
use p2panda_stream::operation::IngestResult;

use crate::topic::LogId;
use crate::{AsBody, testing::AliasedId};
use crate::{polestar as p, timestamp_now};

use super::*;

pub type OpStore = p2panda_store::MemoryStore<LogId, Extensions>;

impl Node {
    #[tracing::instrument(skip_all)]
    pub(super) async fn author_operation(
        &self,
        topic: Topic,
        payload: Payload,
        alias: Option<&str>,
    ) -> Result<Header<Extensions>, anyhow::Error> {
        self.author_operation_with_deps(topic, payload, vec![], alias)
            .await
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn author_operation_with_deps(
        &self,
        topic: Topic,
        payload: Payload,
        mut deps: Vec<p2panda_core::Hash>,
        alias: Option<&str>,
    ) -> Result<Header<Extensions>, anyhow::Error> {
        let mut sd = self.space_dependencies.write().await;
        let (ids, space_deps): (Vec<OperationId>, Vec<Hash>) = match &payload {
            Payload::Chat(msgs) => {
                let ids = msgs.iter().map(|msg| msg.id()).collect::<Vec<_>>();
                let deps = msgs
                .iter()
                .flat_map(|msg| {
                        tracing::debug!(
                            id = msg.id().alias(),
                            argtype = ?msg.arg_type(),
                            batch = ?ids.iter().map(|id| id.alias()).collect::<Vec<_>>(),
                            deps = ?msg.dependencies().iter().map(|id| id.alias()).collect::<Vec<_>>(),
                            "authoring space msg",
                        );
                        msg.dependencies()
                    })
                    .flat_map(|dep| match sd.get(&dep) {
                        Some(hash) => Some(hash.clone()),
                        None => {
                            // If the msg is part of the set being committed, it's ok
                            if !ids.contains(&dep) {
                                tracing::error!(
                                    dep = dep.alias(),
                                    deps = ?sd.keys().map(|k| k.alias()).collect::<Vec<_>>(),
                                    ids = ?ids.iter().map(|id| id.alias()).collect::<Vec<_>>(),
                                    "space dep should have been seen already"
                                );
                                panic!("space dep should have been seen already")
                            }
                            None
                        }
                    })
                    .collect();
                (ids, deps)
            }
            Payload::Announcements(_) => (vec![], vec![]),
            Payload::Inbox(_) => (vec![], vec![]),
            Payload::Private(_) => (vec![], vec![]),
        };

        deps.extend(space_deps.into_iter());

        let operation = create_operation(
            &self.op_store,
            &self.private_key,
            topic.clone(),
            payload.clone(),
            deps.clone(),
        )
        .await?;
        let Operation { header, body, hash } = operation.clone();

        if let Some(alias) = alias {
            header.hash().aliased(alias);
        }

        {
            let space_msgs = match &payload {
                Payload::Chat(msgs) => msgs.iter().map(|m| m.id().alias()).collect::<Vec<_>>(),
                Payload::Inbox(_) => vec![],
                Payload::Announcements(_) => vec![],
                Payload::Private(_) => vec![],
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
                self.process_ordering(topic, operation).await?;

                self.process_operation(topic, op, self.author_store.clone(), true)
                    .await?;

                // self.notify_payload(&header, &payload).await?;
                tracing::debug!(?topic, hash = hash.alias(), "authored operation");

                p::emit(p::Action::AuthorOp {
                    topic,
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

        match self.gossip.read().await.get(&topic) {
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

pub(crate) async fn create_operation(
    store: &OpStore,
    private_key: &PrivateKey,
    topic: Topic,
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
