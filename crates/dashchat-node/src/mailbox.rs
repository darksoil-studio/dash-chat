pub mod mem;

use std::time::Duration;

use named_id::Rename;
use p2panda_core::Body;
use tokio::sync::mpsc;
use tracing::Instrument;

use crate::{DeviceId, Header, Operation, topic::LogId};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const RELAY_FETCH_INTERVAL: Duration = Duration::from_secs(3);
const RELAY_ERROR_INTERVAL: Duration = Duration::from_secs(15);

#[async_trait::async_trait]
pub trait MailboxClient<Op>: Clone + Send + Sync + 'static {
    /// Publish an operation to the mailbox for the given topic.
    async fn publish(&self, topic: LogId, op: Op) -> Result<(), anyhow::Error>;

    /// Fetch operations from the mailbox for the given topic.
    /// The implementation is expected to return only operations that were not previously fetched,
    /// though duplicates will be tolerated.
    async fn fetch(&self, topic: LogId) -> Result<Vec<Op>, anyhow::Error>;

    /// Touch the mailbox to indicate that the given topic has been subscribed to.
    /// Returns true if the topic was not previously touched, false otherwise.
    async fn touch(&self, topic: LogId) -> bool;

    async fn add_author(&self, topic: LogId, author: DeviceId) -> Result<(), anyhow::Error>;
}

/// Subscription can only be implemented for a mailbox that returns Operation-equivalent items.
#[async_trait::async_trait]
pub trait MailboxSubscription {
    async fn subscribe(
        &self,
        topic: LogId,
    ) -> Result<Option<mpsc::Receiver<Operation>>, anyhow::Error>;
}

#[async_trait::async_trait]
impl<T> MailboxSubscription for T
where
    T: MailboxClient<MailboxOperation>,
{
    async fn subscribe(
        &self,
        topic: LogId,
    ) -> Result<Option<mpsc::Receiver<Operation>>, anyhow::Error> {
        if !self.touch(topic).await {
            tracing::warn!(topic = ?topic.renamed(), "mailbox already subscribed");
            return Ok(None);
        }
        tracing::info!(topic = ?topic.renamed(), "subscribing to mailbox");
        let (tx, rx) = mpsc::channel(100);
        let mailbox = self.clone();
        tokio::spawn(
            async move {
                loop {
                    match mailbox.fetch(topic).await {
                        Ok(ops) => {
                            for op in ops {
                                mailbox
                                    .add_author(topic, op.header.public_key.into())
                                    .await
                                    .unwrap();
                                tx.send(op.into()).await.unwrap();
                            }
                            tokio::time::sleep(RELAY_FETCH_INTERVAL).await;
                        }
                        Err(err) => {
                            tracing::error!(?err, "fetch mailbox error");
                            tokio::time::sleep(RELAY_ERROR_INTERVAL).await;
                        }
                    }
                }
            }
            .instrument(tracing::info_span!("mailbox subscription")),
        );
        Ok(Some(rx))
    }
}

pub trait MailboxItem: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    fn hash(&self) -> p2panda_core::Hash;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MailboxOperation {
    pub header: Header,
    pub body: Option<Body>,
}

impl MailboxItem for MailboxOperation {
    fn hash(&self) -> p2panda_core::Hash {
        self.header.hash()
    }
}

impl MailboxItem for bytes::Bytes {
    fn hash(&self) -> p2panda_core::Hash {
        p2panda_core::Hash::new(self)
    }
}
