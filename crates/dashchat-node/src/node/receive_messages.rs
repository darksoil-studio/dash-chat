use std::collections::HashSet;
use std::time::Duration;

use p2panda_core::Hash;

use super::*;
use crate::payload::{ChatPayload, DeviceGroupPayload, Payload};
use crate::topic::TopicId;

impl Node {
    /// Get all contacts from the device group topic.
    async fn get_contacts(&self) -> anyhow::Result<Vec<AgentId>> {
        let ids = self
            .get_interleaved_logs(self.device_group_topic().into(), vec![self.device_id()])
            .await?
            .into_iter()
            .filter_map(|(_, payload)| match payload {
                Some(Payload::DeviceGroup(DeviceGroupPayload::AddContact(qr))) => Some(qr.agent_id),
                _ => None,
            })
            .collect();
        Ok(ids)
    }

    /// Find peer messages in a direct chat topic that don't have a corresponding ReceivedMessages operation from me.
    async fn find_unacknowledged_messages(&self, topic_id: TopicId) -> anyhow::Result<Vec<Hash>> {
        let my_device_id = self.device_id();
        let authors = self.get_authors(topic_id).await?;

        // Collect all message hashes from peers (not from me)
        let mut peer_message_hashes: HashSet<Hash> = HashSet::new();
        for author in &authors {
            if *author == my_device_id {
                continue;
            }
            for (header, body) in self.get_log(topic_id, *author).await? {
                if let Some(body) = body {
                    if let Ok(Payload::Chat(ChatPayload::Message(_))) = Payload::try_from_body(&body)
                    {
                        peer_message_hashes.insert(header.hash());
                    }
                }
            }
        }

        // Collect all message hashes I've already acknowledged
        let mut acknowledged_hashes: HashSet<Hash> = HashSet::new();
        for (_, body) in self.get_log(topic_id, my_device_id).await? {
            if let Some(body) = body {
                if let Ok(Payload::Chat(ChatPayload::ReceivedMessages(hashes))) =
                    Payload::try_from_body(&body)
                {
                    acknowledged_hashes.extend(hashes);
                }
            }
        }

        // Return unacknowledged messages
        let unacknowledged: Vec<Hash> = peer_message_hashes
            .difference(&acknowledged_hashes)
            .cloned()
            .collect();

        Ok(unacknowledged)
    }

    /// Create ReceivedMessages operation for unacknowledged messages in a topic.
    pub async fn acknowledge_received_messages(&self, topic_id: TopicId) -> anyhow::Result<()> {
        let unacked = self.find_unacknowledged_messages(topic_id).await?;
        if unacked.is_empty() {
            return Ok(());
        }

        tracing::debug!(
            topic = ?topic_id.renamed(),
            count = unacked.len(),
            "acknowledging received messages"
        );

        self.author_operation(
            Topic::untyped(*topic_id),
            Payload::Chat(ChatPayload::ReceivedMessages(unacked)),
            Some("received_messages"),
        )
        .await?;

        Ok(())
    }

    /// Spawn background task that checks all direct chat topics for unacknowledged messages every 500ms.
    pub fn spawn_receipt_background_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;

                // Get all contacts and their direct chat topics
                let contacts = match self.get_contacts().await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!(?e, "failed to get contacts for receipt acknowledgment");
                        continue;
                    }
                };

                for contact in contacts {
                    let topic_id: TopicId = self.direct_chat_topic(contact).into();
                    if let Err(e) = self.acknowledge_received_messages(topic_id).await {
                        tracing::warn!(
                            ?e,
                            contact = ?contact.renamed(),
                            "failed to acknowledge received messages"
                        );
                    }
                }
            }
        });
    }
}
