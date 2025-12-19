pub mod mem;

use std::time::Duration;

use p2panda_core::Body;
use tokio::sync::mpsc;

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
}

pub trait RelaySubscription {
    fn subscribe(&self, topic: LogId) -> Result<mpsc::Receiver<Operation>, anyhow::Error>;
}

impl<T> RelaySubscription for T
where
    T: RelayClient<RelayOperation>,
{
    fn subscribe(&self, topic: LogId) -> Result<mpsc::Receiver<Operation>, anyhow::Error> {
        let (tx, rx) = mpsc::channel(100);
        let relay = self.clone();
        tokio::spawn(async move {
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
        });
        Ok(rx)
    }
}

pub trait RelayBlob: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {}
impl<T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static> RelayBlob for T {}

#[derive(Clone, Serialize, Deserialize)]
pub struct RelayOperation {
    pub header: Header,
    pub body: Option<Body>,
}
