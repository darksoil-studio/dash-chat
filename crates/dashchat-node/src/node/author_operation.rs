use p2panda_core::{Hash, Operation};
use p2panda_spaces::{OperationId, traits::AuthoredMessage};
use p2panda_stream::operation::IngestResult;

use crate::spaces::SpaceOperation;
use crate::topic::{LogId, TopicKind};
use crate::{AsBody, testing::AliasedId};
use crate::{polestar as p, timestamp_now};

use super::*;

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
        let (header, body) = self
            .op_store
            .create_operation(
                &self.local_data.private_key,
                topic.clone(),
                payload.clone(),
                vec![],
            )
            .await?;

        self.process_authored_operation(header, body, alias).await
    }

    // Can't do this unless we allow multiple operations to be created but not yet ingested.
    pub(crate) async fn process_authored_space_messages(
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

        tracing::info!(
            ?log_id,
            hash = hash.alias(),
            seq_num = header.seq_num,
            "PUB: authoring operation"
        );

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
                tracing::error!(
                    ?log_id,
                    hash = hash.alias(),
                    ?backlink,
                    ?missing,
                    "operation could not be ingested"
                );
                panic!("operation could not be ingested, check your sequence numbers!");
            }
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
