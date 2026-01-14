pub mod manager;
pub mod mem;
pub mod toy;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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
    async fn publish(&self, ops: Vec<Op>) -> Result<(), anyhow::Error>;

    /// Fetch operations from the mailbox for the given topics.
    ///
    /// The inner map associated each author with the height of their locally stored log.
    /// The height represents the highest sequence number stored for that author, meaning that the mailbox
    /// should only return operations with a higher sequence for that author.
    /// NOTE that this is a subtractive, not additive, filter, meaning that any authors not included
    /// in the `min_heights` list will have their *entire* log returned, including if `min_heights` is empty.
    /// This is so that the mailbox is used for author discovery as well.
    /// The intention is that all data is encrypted and only decipherable by valid recipients.
    async fn fetch(&self, request: FetchRequest<Op>) -> Result<FetchResponse<Op>, anyhow::Error>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(deserialize = "Op: DeserializeOwned"))]
pub struct FetchRequest<Op: MailboxItem>(pub BTreeMap<LogId, FetchTopicRequest<Op>>);

pub type FetchTopicRequest<Op> = BTreeMap<<Op as MailboxItem>::Author, u64>;

/// Returned by the `fetch` method.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(deserialize = "Op: DeserializeOwned"))]
pub struct FetchResponse<Op: MailboxItem>(pub BTreeMap<LogId, FetchTopicResponse<Op>>);

/// Returned by the `fetch` method.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(deserialize = "Op: DeserializeOwned"))]
pub struct FetchTopicResponse<Op: MailboxItem> {
    /// The operations not held locally that were fetched.
    pub ops: Vec<Op>,
    /// The operations held locally that are missing from the mailbox,
    /// and which this node should now publish.
    pub missing: HashMap<<Op as MailboxItem>::Author, Vec<u64>>,
}

pub trait MailboxItem: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    type Hash: Copy
        + Eq
        + Ord
        + std::hash::Hash
        + Rename
        + Serialize
        + DeserializeOwned
        + Send
        + Sync;
    type Author: Copy
        + Eq
        + Ord
        + std::hash::Hash
        + Rename
        + Serialize
        + DeserializeOwned
        + Send
        + Sync;

    fn hash(&self) -> Self::Hash;
    fn author(&self) -> Self::Author;
    fn seq_num(&self) -> u64;
    fn topic(&self) -> LogId;
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

    fn topic(&self) -> LogId {
        self.header.extensions.log_id
    }
}
