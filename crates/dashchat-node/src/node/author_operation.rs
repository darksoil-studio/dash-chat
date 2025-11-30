use p2panda_core::{Hash, Operation};
use p2panda_spaces::{OperationId, traits::AuthoredMessage};
use p2panda_stream::operation::IngestResult;

use crate::spaces::SpaceOperation;
use crate::topic::{LogId, TopicKind};
use crate::{AsBody, testing::AliasedId};
use crate::{polestar as p, timestamp_now};

use super::*;

impl Node {
    pub(super) async fn author_operation<K: TopicKind>(
        &self,
        topic: Topic<K>,
        payload: Payload,
        alias: Option<&str>,
    ) -> Result<Header, anyhow::Error> {
        let (header, body) = self
            .op_store
            .author_operation(
                &self.local_data.private_key,
                topic.clone(),
                payload.clone(),
                vec![],
                alias,
            )
            .await?;

        let op = Operation {
            hash: header.hash(),
            header,
            body,
        };
        self.process_authored_ingested_operation(op).await
    }

    // Can't do this unless we allow multiple operations to be created but not yet ingested.
    pub(crate) async fn process_authored_ingested_space_messages(
        &self,
        ops: Vec<SpaceOperation>,
    ) -> Result<(), anyhow::Error> {
        for op in ops {
            let op = op.into_operation()?;
            self.process_authored_ingested_operation(op).await?;
        }
        Ok(())
    }

    pub(crate) async fn process_authored_ingested_operation(
        &self,
        op: Operation<Extensions>,
    ) -> Result<Header, anyhow::Error> {
        let log_id = op.header.extensions.log_id;
        self.process_operation(op.clone(), self.author_store.clone(), true, false)
            .await?;
        let Operation { header, body, hash } = op;

        // self.notify_payload(&header, &payload).await?;
        tracing::debug!(?log_id, hash = hash.alias(), "authored operation");

        match self.initialized_topics.read().await.get(&log_id) {
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
