use p2panda_core::{Hash, Operation};
use p2panda_spaces::{OperationId, traits::AuthoredMessage};
use p2panda_stream::operation::IngestResult;

use crate::spaces::SpaceOperation;
use crate::topic::{LogId, TopicKind};
use crate::{AsBody, testing::AliasedId};
use crate::{polestar as p, timestamp_now};

use super::*;

pub type OpStore = p2panda_store::MemoryStore<LogId, Extensions>;

impl Node {
    /// Author a space repair operation without processing it
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
    #[deprecated(note = "use author_operation instead")]
    pub(super) async fn author_repair_operation<K: TopicKind>(
        &self,
        topic: Topic<K>,
        payload: Payload,
        alias: Option<&str>,
    ) -> Result<Header, anyhow::Error> {
        self.author_operation(topic, payload, alias).await
    }

    pub(super) async fn author_operation<K: TopicKind>(
        &self,
        topic: Topic<K>,
        payload: Payload,
        alias: Option<&str>,
    ) -> Result<Header, anyhow::Error> {
        let (header, body) = create_operation(
            &self.op_store,
            &self.local_data.private_key,
            topic.clone(),
            payload.clone(),
            vec![],
        )
        .await?;

        self.process_authored_operation(header, body, alias).await
    }

    pub(super) async fn process_authored_space_messages(
        &self,
        ops: Vec<SpaceOperation>,
    ) -> Result<(), anyhow::Error> {
        for op in ops {
            let op = op.into_operation()?;
            self.process_authored_operation(op.header.clone(), op.body, None)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn process_authored_operation(
        &self,
        header: Header,
        body: Option<Body>,
        alias: Option<&str>,
    ) -> Result<Header, anyhow::Error> {
        let log_id = header.extensions.log_id;
        let hash = header.hash();

        if let Some(alias) = alias {
            header.hash().aliased(alias);
        }

        let result = p2panda_stream::operation::ingest_operation(
            &mut *self.op_store.clone(),
            header.clone(),
            body.clone(),
            header.to_bytes(),
            &log_id.into(),
            false,
        )
        .await?;

        match result {
            IngestResult::Complete(op @ Operation { hash: hash2, .. }) => {
                assert_eq!(hash, hash2);

                self.process_ordering(op.clone()).await?;

                // NOTE: if we fail to process here, incoming operations will be stuck as pending!
                self.process_operation(op, self.author_store.clone(), true, false)
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
                    ?log_id,
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

        match self
            .initialized_topics
            .read()
            .await
            .get(&header.extensions.log_id)
        {
            Some(gossip) => {
                gossip
                    .send(ToNetwork::Message {
                        bytes: encode_gossip_message(&header, body.as_ref())?,
                    })
                    .await?;
            }
            None => {
                tracing::error!(?log_id, "no gossip channel found for log id");
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
) -> Result<(Header, Option<Body>), anyhow::Error> {
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

    Ok((header, body))
}
