use p2panda_store::OperationStore;

use crate::stores::OpStore;

use super::*;

#[derive(Clone, Debug)]
pub struct MailboxesConfig {
    pub success_interval: Duration,
    pub error_interval: Duration,
}

impl Default for MailboxesConfig {
    fn default() -> Self {
        Self {
            success_interval: Duration::from_secs(5),
            error_interval: Duration::from_secs(15),
        }
    }
}

#[derive(Clone)]
pub struct Mailboxes<S>
where
    S: OperationStore<TopicId, Extensions> + LogStore<TopicId, Extensions> + Send + Sync + 'static,
{
    mailboxes: Arc<Mutex<Vec<Arc<dyn MailboxClient>>>>,
    topics: Arc<Mutex<HashMap<TopicId, mpsc::Sender<Operation>>>>,
    store: OpStore<S>,
    config: MailboxesConfig,
    trigger: mpsc::Sender<()>,
    // TODO: add a receiver to short circuit the polling interval
}

impl<S> Mailboxes<S>
where
    S: OperationStore<TopicId, Extensions> + LogStore<TopicId, Extensions> + Send + Sync + 'static,
{
    fn new(store: OpStore<S>, config: MailboxesConfig, trigger: mpsc::Sender<()>) -> Self {
        Self {
            mailboxes: Arc::new(Mutex::new(Default::default())),
            topics: Arc::new(Mutex::new(Default::default())),
            store,
            config,
            trigger,
        }
    }

    pub async fn add(&self, mailbox: impl MailboxClient) {
        self.mailboxes.lock().await.push(Arc::new(mailbox));
    }

    pub async fn clear(&self) {
        self.mailboxes.lock().await.clear();
    }

    pub async fn subscribed_topics(&self) -> BTreeSet<TopicId> {
        self.topics.lock().await.keys().cloned().collect()
    }

    pub async fn trigger_sync(&self) {
        if let Err(e) = self.trigger.send(()).await {
            tracing::error!(?e, "failed to send early trigger to mailbox manager");
        }
    }

    pub async fn subscribe(
        &self,
        topic: TopicId,
    ) -> Result<mpsc::Receiver<Operation>, anyhow::Error> {
        tracing::info!(topic = ?topic.renamed(), "subscribing to topic");
        let (tx, rx) = mpsc::channel(100);
        self.topics.lock().await.insert(topic, tx);
        Ok(rx)
    }

    pub async fn unsubscribe(&self, topic: TopicId) -> Result<(), anyhow::Error> {
        tracing::info!(topic = ?topic.renamed(), "unsubscribing from topic");
        self.topics.lock().await.remove(&topic);
        Ok(())
    }

    pub async fn spawn(store: OpStore<S>, config: MailboxesConfig) -> Result<Self, anyhow::Error> {
        let (trigger_tx, mut trigger_rx) = mpsc::channel(1);
        let manager = Self::new(store, config, trigger_tx);
        let r = manager.clone();
        tokio::spawn(
            async move {
                let mut next_mailbox = 0;
                let mut interval = tokio::time::Duration::ZERO;
                // The two match conditions are:
                // - Ok(Some(())): a trigger was received
                // - Err(_): the timeout elapsed
                while let Ok(Some(())) | Err(_) =
                    tokio::time::timeout(interval, trigger_rx.recv()).await
                {
                    (interval, next_mailbox) = manager.one_iteration(next_mailbox).await;
                }

                #[allow(unused)]
                {
                    tracing::warn!("poll mailboxes loop exited");
                }
            }
            .instrument(tracing::info_span!("poll mailboxes")),
        );

        Ok(r)
    }

    async fn one_iteration(&self, mut mailbox_index: usize) -> (tokio::time::Duration, usize) {
        mailbox_index += 1;
        let mailbox = {
            let mm = self.mailboxes.lock().await;
            if mailbox_index >= mm.len() {
                mailbox_index = 0;
            }

            match mm.get(mailbox_index) {
                Some(mailbox) => mailbox.clone(),
                None => {
                    tracing::warn!("empty mailbox list, no mailbox to fetch from");
                    return (self.config.error_interval, mailbox_index);
                }
            }
        };
        tracing::trace!("polling mailbox {mailbox_index}");

        let topics = self.subscribed_topics().await;
        if topics.is_empty() {
            tracing::warn!("no topics subscribed, nothing to fetch this interval");
            return (self.config.error_interval, mailbox_index);
        }

        match self.sync_topics(topics.into_iter(), mailbox.clone()).await {
            Ok(()) => {
                return (self.config.success_interval, mailbox_index);
            }
            Err(err) => {
                tracing::error!(?err, "fetch mailbox error");
                return (self.config.error_interval, mailbox_index);
            }
        }
    }

    /// Immediately sync the given topics with the given mailbox:
    /// - Ensure all ops held by the mailbox are fetched
    /// - Publish any ops that the mailbox is missing to the mailbox
    pub async fn sync_topics(
        &self,
        topics: impl Iterator<Item = TopicId>,
        mailbox: Arc<dyn MailboxClient>,
    ) -> anyhow::Result<()> {
        let mut request = BTreeMap::new();
        for topic in topics {
            let heights =
                BTreeMap::from_iter(self.store.get_log_heights(&topic).await?.into_iter());
            request.insert(topic, heights);
        }

        let FetchResponse(response) = mailbox.fetch(FetchRequest(request)).await?;

        let mut ops_to_publish = vec![];
        for (topic, response) in response.into_iter() {
            let FetchTopicResponse { ops, missing } = response;
            tracing::info!(
                ops = ops.len(),
                missing = missing.len(),
                "fetched operations"
            );

            let Some(sender) = self.topics.lock().await.get(&topic).cloned() else {
                tracing::warn!(topic = ?topic.renamed(), "no sender for topic");
                continue;
            };

            for op in ops {
                sender.send(op.into()).await?;
            }

            for (author, seqs) in missing {
                let Some(lowest) = seqs.iter().min() else {
                    continue;
                };
                let Some(log) = self
                    .store
                    .get_log(&author, &topic, Some(*lowest))
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to get log for {topic:?}: {err}"))?
                else {
                    continue;
                };

                for seq in &seqs {
                    // The operations in the 0..lowest range are not included in the log vector,
                    // because `get_log()` is called with `lowest` as the starting point.
                    // Adjust the index to take this into account:
                    let index = seq - lowest;
                    if let Some((header, body)) = log.get(index as usize) {
                        let op = MailboxOperation {
                            header: header.clone(),
                            body: body.clone(),
                        };
                        ops_to_publish.push(op);
                    }
                }
            }
        }

        mailbox.publish(ops_to_publish).await?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use std::time::Duration;

    use crate::{mailbox::mem::MemMailbox, testing::*, *};

    /// Very simple test which circumvents the contact adding system:
    /// - alice sends a message to a direct chat topic
    /// - alice and bobbi add a mailbox after the fact
    /// - bobbi still gets the message later
    #[tokio::test(flavor = "multi_thread")]
    async fn test_mailbox_late_join() {
        dashchat_node::testing::setup_tracing(
            &[
                "dashchat=info",
                "p2panda_stream=warn",
                "p2panda_auth=warn",
                "p2panda_encryption=warn",
                "p2panda_spaces=warn",
                "named_id=warn",
            ],
            true,
        );

        let mb = MemMailbox::new();
        let config = NodeConfig::testing();

        // Start with no mailbox
        let alice = TestNode::new(config.clone(), Some("alice")).await;
        let bobbi = TestNode::new(config.clone(), Some("bobbi")).await;

        let chat = alice.direct_chat_topic(bobbi.agent_id());
        alice.initialize_topic(chat, false).await.unwrap();
        alice.send_message(chat, "Hello".into()).await.unwrap();

        println!("=== adding mailboxes ===");
        bobbi.add_mailbox_client(mb.client()).await;
        alice.add_mailbox_client(mb.client()).await;

        bobbi.initialize_topic(chat, false).await.unwrap();
        println!("=== added mailboxes ===");

        wait_for(
            Duration::from_millis(100),
            Duration::from_secs(5),
            || async {
                (bobbi.get_messages(chat).await.unwrap().len() == 1).ok_or("message not received")
            },
        )
        .await
        .unwrap();
    }
}
