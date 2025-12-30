use bytes::Bytes;

use crate::mailbox::MailboxClient;
use crate::polestar as p;
use crate::spaces::SpaceOperation;
use crate::topic::TopicKind;

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
            hash: header.hash().with_serial(),
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
        op: Operation,
    ) -> Result<Header, anyhow::Error> {
        let log_id = op.header.extensions.log_id;
        op.hash.with_serial();
        self.process_operation(op.clone(), true, false).await?;
        let Operation { header, body, hash } = op;

        // self.notify_payload(&header, &payload).await?;
        tracing::debug!(?log_id, hash = ?hash.renamed(), "authored operation");

        self.mailbox
            .publish(log_id.into(), (header.clone(), body).into())
            .await?;

        #[cfg(feature = "p2p")]
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
