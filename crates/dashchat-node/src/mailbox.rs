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

const MAILBOX_FETCH_INTERVAL: Duration = Duration::from_secs(3);
const MAILBOX_ERROR_INTERVAL: Duration = Duration::from_secs(15);

#[async_trait::async_trait]
pub trait MailboxClient<Op: MailboxItem = MailboxOperation>: Send + Sync + 'static {
    /// Publish an operation to the mailbox for the given topic.
    async fn publish(&self, topic: LogId, ops: Vec<Op>) -> Result<(), anyhow::Error>;

    /// Fetch operations from the mailbox for the given topic.
    /// The implementation is expected to return only operations that were not previously fetched,
    /// though duplicates will be tolerated.
    async fn fetch(
        &self,
        topic: LogId,
        min_heights: &[(Op::Author, u64)],
    ) -> Result<FetchResponse<Op>, anyhow::Error>;
}

/// Returned by the `fetch` method.
pub struct FetchResponse<Op: MailboxItem> {
    /// The operations not held locally that were fetched.
    pub ops: Vec<Op>,
    /// The operations held locally that are missing from the mailbox,
    /// and which this node should now publish.
    pub missing: HashMap<<Op as MailboxItem>::Author, Vec<u64>>,
}

pub trait MailboxItem: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    type Hash: Copy + Eq + std::hash::Hash + Rename + Send + Sync;
    type Author: Copy + Eq + std::hash::Hash + Rename + Send + Sync;

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

#[derive(Clone)]
pub struct Mailboxes(Arc<Mutex<HashMap<(), Arc<dyn MailboxClient>>>>);

#[async_trait::async_trait]
impl MailboxClient for Mailboxes {
    async fn publish(&self, topic: LogId, ops: Vec<MailboxOperation>) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn fetch(
        &self,
        topic: LogId,
        min_heights: &[(DeviceId, u64)],
    ) -> Result<FetchResponse<MailboxOperation>, anyhow::Error> {
        todo!()
    }
}

impl Mailboxes {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub async fn add(&self, mailbox: impl MailboxClient) {
        self.0.lock().await.insert((), Arc::new(mailbox));
    }

    pub async fn subscribed_topics(&self) -> BTreeSet<LogId> {
        todo!()
    }

    pub async fn subscribe(
        &self,
        topic: LogId,
        logs: impl LogStore<LogId, Extensions> + Send + Sync + 'static,
    ) -> Result<Option<mpsc::Receiver<Operation>>, anyhow::Error> {
        // TODO: fix
        // if !self.touch(topic).await {
        //     tracing::warn!(topic = ?topic.renamed(), "mailbox already subscribed");
        //     return Ok(None);
        // }
        tracing::info!(topic = ?topic.renamed(), "subscribing to mailbox");
        let (tx, rx) = mpsc::channel(100);
        let mailbox = self.clone();
        tokio::spawn(
            async move {
                loop {
                    let heights = match logs.get_log_heights(&topic).await {
                        Ok(heights) => heights,
                        Err(err) => {
                            tracing::error!(?err, "failed to get log heights for {topic:?}, terminating subscription loop");
                            break;
                        }
                    };
                    let min_heights = heights
                        .into_iter()
                        .map(|(pk, height)| (DeviceId::from(pk), height))
                        .collect::<Vec<_>>();
                    match mailbox.fetch(topic, &min_heights).await {
                        Ok(ops) => {
                            // TODO: also handle `missing`
                            for op in ops.ops {
                                tx.send(op.into()).await.unwrap();
                            }
                            tokio::time::sleep(MAILBOX_FETCH_INTERVAL).await;
                        }
                        Err(err) => {
                            tracing::error!(?err, "fetch mailbox error");
                            tokio::time::sleep(MAILBOX_ERROR_INTERVAL).await;
                        }
                    }
                }
            }
            .instrument(tracing::info_span!("mailbox subscription")),
        );
        Ok(Some(rx))
    }
}
