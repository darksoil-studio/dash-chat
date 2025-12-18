pub mod mem;

use bytes::Bytes;

use crate::topic::LogId;

#[async_trait::async_trait]
pub trait RelayClient {
    /// Publish an operation to the relay for the given topic.
    async fn publish(&self, topic: LogId, op: Bytes) -> Result<(), anyhow::Error>;

    /// Fetch operations from the relay for the given topic.
    /// The implementation is expected to return only operations that were not previously fetched,
    /// though duplicates will be tolerated.
    async fn fetch(&self, topic: LogId) -> Result<Vec<Bytes>, anyhow::Error>;
}
