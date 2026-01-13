pub mod manager;
pub mod mem;

use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
    time::Duration,
};

use named_id::Rename;
use p2panda_core::Body;
use p2panda_store::LogStore;
use tokio::sync::{Mutex, mpsc};
use tracing::Instrument;

use crate::{DeviceId, Extensions, Header, Operation, topic::LogId};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[async_trait::async_trait]
pub trait MailboxClient<Op: MailboxItem = MailboxOperation>: Send + Sync + 'static {
    /// Publish an operation to the mailbox for the given topic.
    async fn publish(&self, topic: LogId, ops: Vec<Op>) -> Result<(), anyhow::Error>;

    /// Fetch operations from the mailbox for the given topic.
    ///
    /// The `min_heights` parameter is a list of (author, log_height) pairs.
    /// The height represents the highest sequence number stored for that author, meaning that the mailbox
    /// should only return operations with a higher sequence for that author.
    /// NOTE that this is a subtractive, not additive, filter, meaning that any authors not included
    /// in the `min_heights` list will have their *entire* log returned, including if `min_heights` is empty.
    /// This is so that the mailbox is used for author discovery as well.
    /// The intention is that all data is encrypted and only decipherable by valid recipients.
    async fn fetch(
        &self,
        topic: LogId,
        min_heights: &[(Op::Author, u64)],
    ) -> Result<FetchResponse<Op>, anyhow::Error>;
}

/// Returned by the `fetch` method.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(deserialize = "Op: DeserializeOwned"))]
pub struct FetchResponse<Op: MailboxItem> {
    /// The operations not held locally that were fetched.
    pub ops: Vec<Op>,
    /// The operations held locally that are missing from the mailbox,
    /// and which this node should now publish.
    pub missing: HashMap<<Op as MailboxItem>::Author, Vec<u64>>,
}

pub trait MailboxItem: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    type Hash: Copy + Eq + std::hash::Hash + Rename + Serialize + DeserializeOwned + Send + Sync;
    type Author: Copy + Eq + std::hash::Hash + Rename + Serialize + DeserializeOwned + Send + Sync;

    fn hash(&self) -> Self::Hash;
    fn author(&self) -> Self::Author;
    fn seq_num(&self) -> u64;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MailboxOperation {
    pub header: Header,
    pub body: Option<Body>,
}

impl MailboxItem for MailboxOperation {
    type Hash = p2panda_core::Hash;
    type Author = DeviceId;

    fn hash(&self) -> p2panda_core::Hash {
        self.header.hash()
    }

    fn author(&self) -> DeviceId {
        self.header.public_key.into()
    }

    fn seq_num(&self) -> u64 {
        self.header.seq_num
    }
}
