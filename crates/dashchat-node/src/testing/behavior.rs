//! Simulate a node's user interaction.

use std::time::Duration;

use anyhow::Context;

use super::*;
use crate::*;

#[derive(derive_more::Deref)]
pub struct Behavior {
    #[deref]
    node: TestNode,
    pub watcher: Watcher<Notification>,
}

impl Behavior {
    pub fn new(node: TestNode, watcher: Watcher<Notification>) -> Self {
        Self { node, watcher }
    }

    #[tracing::instrument(skip_all, fields(me = ?self.node.public_key()))]
    pub async fn initiate_and_establish_contact(
        &mut self,
        other: &mut Behavior,
        share_intent: ShareIntent,
    ) -> anyhow::Result<()> {
        let qr = self.new_qr_code(share_intent, true).await?;
        other.add_contact(qr).await?;
        self.accept_next_contact().await?;
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(me = ?self.node.public_key()))]
    pub async fn accept_next_contact(&mut self) -> anyhow::Result<QrCode> {
        let qr = self
            .watcher
            .watch_mapped(Duration::from_secs(5), |n: &Notification| {
                tracing::debug!(
                    hash = n.header.hash().alias(),
                    "checking for contact invitation"
                );
                let Payload::Inbox(InboxPayload::Contact(qr)) = &n.payload else {
                    return None;
                };
                Some(qr.clone())
            })
            .await
            .context("no contact invitation found")?;

        self.node.add_contact(qr.clone()).await?;
        Ok(qr)
    }

    #[tracing::instrument(skip_all, fields(me = ?self.node.public_key()))]
    pub async fn accept_next_group_invitation(&mut self) -> anyhow::Result<ChatId> {
        let chat_id = self
            .watcher
            .watch_mapped(Duration::from_secs(5), |n: &Notification| {
                tracing::debug!(
                    hash = n.header.hash().alias(),
                    "checking for group invitation"
                );
                let Payload::Chat(ChatPayload::JoinGroup(chat_id)) = &n.payload else {
                    return None;
                };
                Some(*chat_id)
            })
            .await
            .context("no group invitation found")?;

        self.node.join_group(chat_id).await?;
        Ok(chat_id)
    }
}
