pub mod mem;

use std::time::Duration;

use p2panda_core::Body;
use tokio::sync::mpsc;
use tracing::Instrument;

use crate::{Header, Operation, topic::LogId};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const RELAY_FETCH_INTERVAL: Duration = Duration::from_secs(3);
const RELAY_ERROR_INTERVAL: Duration = Duration::from_secs(15);

#[async_trait::async_trait]
pub trait RelayClient<Op>: Clone + Send + Sync + 'static {
    /// Publish an operation to the relay for the given topic.
    async fn publish(&self, topic: LogId, op: Op) -> Result<(), anyhow::Error>;

    /// Fetch operations from the relay for the given topic.
    /// The implementation is expected to return only operations that were not previously fetched,
    /// though duplicates will be tolerated.
    async fn fetch(&self, topic: LogId) -> Result<Vec<Op>, anyhow::Error>;

    /// Touch the relay to indicate that the given topic has been subscribed to.
    /// Returns true if the topic was not previously touched, false otherwise.
    async fn touch(&self, topic: LogId) -> bool;
}

/// Subscription can only be implemented for a relay that returns Operation-equivalent items.
#[async_trait::async_trait]
pub trait RelaySubscription {
    async fn subscribe(
        &self,
        topic: LogId,
    ) -> Result<Option<mpsc::Receiver<Operation>>, anyhow::Error>;
}

#[async_trait::async_trait]
impl<T> RelaySubscription for T
where
    T: RelayClient<RelayOperation>,
{
    async fn subscribe(
        &self,
        topic: LogId,
    ) -> Result<Option<mpsc::Receiver<Operation>>, anyhow::Error> {
        if !self.touch(topic).await {
            return Ok(None);
        }
        let (tx, rx) = mpsc::channel(100);
        let relay = self.clone();
        tokio::spawn(
            async move {
                loop {
                    match relay.fetch(topic).await {
                        Ok(ops) => {
                            for op in ops {
                                tx.send(op.into()).await.unwrap();
                            }
                            tokio::time::sleep(RELAY_FETCH_INTERVAL).await;
                        }
                        Err(err) => {
                            tracing::error!(?err, "fetch relay error");
                            tokio::time::sleep(RELAY_ERROR_INTERVAL).await;
                        }
                    }
                }
            }
            .instrument(tracing::info_span!("relay subscription")),
        );
        Ok(Some(rx))
    }
}

pub trait RelayItem: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    fn hash(&self) -> p2panda_core::Hash;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RelayOperation {
    pub header: Header,
    pub body: Option<Body>,
}

impl RelayItem for RelayOperation {
    fn hash(&self) -> p2panda_core::Hash {
        self.header.hash()
    }
}

impl RelayItem for bytes::Bytes {
    fn hash(&self) -> p2panda_core::Hash {
        p2panda_core::Hash::new(self)
    }
}
