use super::*;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Arc,
};

use named_id::Rename;
use p2panda_core::Body;
use tokio::sync::{Mutex, RwLock};

use crate::{AgentId, DeviceId, Header, topic::LogId};

impl From<Operation> for MailboxOperation {
    fn from(op: Operation) -> Self {
        Self {
            header: op.header,
            body: op.body,
        }
    }
}

impl From<MailboxOperation> for Operation {
    fn from(op: MailboxOperation) -> Self {
        Self {
            hash: op.header.hash(),
            header: op.header,
            body: op.body,
        }
    }
}

impl From<(Header, Option<Body>)> for MailboxOperation {
    fn from((header, body): (Header, Option<Body>)) -> Self {
        Self { header, body }
    }
}

/// A client for the in-memory mailbox server.
/// This client is stateful, so all requests for a node should go through a single client
/// instance. State is shared between all cloned copies of this.
#[derive(Clone)]
pub struct MemMailboxClient<Op: MailboxItem = MailboxOperation> {
    mailbox: MemMailbox<Op>,
    latest: Arc<Mutex<HashMap<LogId, usize>>>,
    authors: Arc<RwLock<HashMap<LogId, HashSet<DeviceId>>>>,
    agents: Arc<RwLock<HashMap<DeviceId, AgentId>>>,
}

impl<Op: MailboxItem> MemMailboxClient<Op> {
    pub async fn subscribed_topics(&self) -> BTreeSet<LogId> {
        self.latest.lock().await.keys().cloned().collect()
    }

    pub async fn authors(&self, topic: LogId) -> Option<HashSet<DeviceId>> {
        let authors = self.authors.read().await;
        authors.get(&topic).cloned()
    }
}

#[derive(Clone)]
pub struct MemMailbox<Op: MailboxItem = MailboxOperation> {
    ops: Arc<RwLock<HashMap<LogId, Vec<Op>>>>,
}

impl<Op: MailboxItem> MemMailbox<Op> {
    pub fn new() -> Self {
        Self {
            ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn client(&self) -> MemMailboxClient<Op> {
        MemMailboxClient {
            mailbox: self.clone(),
            latest: Arc::new(Mutex::new(HashMap::new())),
            authors: Arc::new(RwLock::new(HashMap::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl<Op: MailboxItem> MailboxClient<Op> for MemMailboxClient<Op> {
    async fn publish(&self, topic: LogId, op: Op) -> Result<(), anyhow::Error> {
        let mut ops = self.mailbox.ops.write().await;
        tracing::info!(topic = ?topic.renamed(), hash = ?op.hash().renamed(), "publishing mailbox operation");
        ops.entry(topic).or_insert_with(Vec::new).push(op.into());
        Ok(())
    }

    async fn fetch(&self, topic: LogId) -> anyhow::Result<Vec<Op>> {
        let ops = self.mailbox.ops.read().await;
        let mut since_lock = self.latest.lock().await;
        let since = *since_lock.get(&topic).unwrap_or(&0);
        if let Some(ops) = ops.get(&topic) {
            if since >= ops.len() {
                return Ok(vec![]);
            }
            since_lock.insert(topic, ops.len());
            let new: Vec<Op> = ops.iter().skip(since).cloned().collect();

            tracing::info!(
                topic = ?topic.renamed(),
                num = new.len(),
                hashes = ?new.iter().map(|op: &Op| op.hash().renamed()).collect::<Vec<_>>(),
                "fetching mailbox operations"
            );

            Ok(new)
        } else {
            Ok(vec![])
        }
    }

    async fn touch(&self, topic: LogId) -> bool {
        let mut latest = self.latest.lock().await;
        match latest.entry(topic) {
            std::collections::hash_map::Entry::Occupied(_) => false,
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(0);
                true
            }
        }
    }

    async fn add_author(&self, topic: LogId, author: DeviceId) -> Result<(), anyhow::Error> {
        let mut authors = self.authors.write().await;
        authors
            .entry(topic)
            .or_insert_with(HashSet::new)
            .insert(author);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::Topic;

    use super::*;

    fn b(s: &'static str) -> Bytes {
        Bytes::from_static(s.as_bytes())
    }

    #[tokio::test]
    async fn test_mem_mailbox() {
        let mailbox = MemMailbox::<Bytes>::new();
        let client = mailbox.client();

        let tt: [LogId; 2] = [Topic::random().into(), Topic::random().into()];

        assert!(client.fetch(tt[0]).await.unwrap().is_empty());
        assert!(client.fetch(tt[1]).await.unwrap().is_empty());

        client.publish(tt[0], b("0")).await.unwrap();
        client.publish(tt[0], b("1")).await.unwrap();

        client.publish(tt[1], b("2")).await.unwrap();
        client.publish(tt[1], b("3")).await.unwrap();

        assert_eq!(client.fetch(tt[0]).await.unwrap(), vec![b("0"), b("1")]);
        assert_eq!(client.fetch(tt[1]).await.unwrap(), vec![b("2"), b("3")]);
        assert!(client.fetch(tt[0]).await.unwrap().is_empty());
        assert!(client.fetch(tt[1]).await.unwrap().is_empty());

        client.publish(tt[0], b("4")).await.unwrap();
        client.publish(tt[0], b("5")).await.unwrap();

        client.publish(tt[1], b("6")).await.unwrap();
        client.publish(tt[1], b("7")).await.unwrap();

        assert_eq!(client.fetch(tt[0]).await.unwrap(), vec![b("4"), b("5")]);
        assert_eq!(client.fetch(tt[1]).await.unwrap(), vec![b("6"), b("7")]);
        assert!(client.fetch(tt[0]).await.unwrap().is_empty());
        assert!(client.fetch(tt[1]).await.unwrap().is_empty());
    }

    // #[tokio::test]
    // async fn test_mem_mailbox_clients() {
    //     let mailbox = MemRelay::new();

    //     let cc = [mailbox.client(), mailbox.client()];

    //     let tt = [Topic::random(), Topic::random(), Topic::random()];

    //     let oo = (0u8..128)
    //         .map(|i| Bytes::from_static(&[i]))
    //         .collect::<Vec<_>>();
    //     // let o1 = (0..64).map(|i| [i; 4]).collect::<Vec<_>>();
    //     // let o2 = (64..128).map(|i| [i; 4]).collect::<Vec<_>>();
    //     // let o3 = (128..192).map(|i| [i; 4]).collect::<Vec<_>>();

    //     for (i, o) in oo.iter().enumerate() {
    //         let c = cc[i % cc.len()].clone();
    //         let t = tt[i % tt.len()].clone();
    //         c.publish(t.into(), o.clone()).await.unwrap();
    //     }

    //     let op = Bytes::from_static(b"test");
    //     mailbox.publish(t1.into(), op).await.unwrap();
    //     let ops = mailbox.fetch(topic, 0).await.unwrap();
    //     assert_eq!(ops.len(), 1);
    //     assert_eq!(ops[0], op);
    // }
}
