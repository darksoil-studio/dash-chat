use super::*;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
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
    subscribed_topics: Arc<RwLock<BTreeSet<LogId>>>,
}

impl<Op: MailboxItem> MemMailboxClient<Op> {
    pub async fn subscribed_topics(&self) -> BTreeSet<LogId> {
        self.subscribed_topics.read().await.clone()
    }
}

pub type MemMailboxLogs<Op> =
    HashMap<LogId, HashMap<<Op as MailboxItem>::Author, BTreeMap<u64, Op>>>;

#[derive(Clone)]
pub struct MemMailbox<Op: MailboxItem = MailboxOperation> {
    ops: Arc<RwLock<MemMailboxLogs<Op>>>,
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
            subscribed_topics: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

#[async_trait::async_trait]
impl<Op: MailboxItem> MailboxClient<Op> for MemMailboxClient<Op> {
    async fn publish(&self, topic: LogId, ops: Vec<Op>) -> Result<(), anyhow::Error> {
        let mut store = self.mailbox.ops.write().await;
        // ops.entry(topic).or_insert_with(Vec::new).push(op.into());
        for op in ops {
            let author = op.author();
            let seq_num = op.seq_num();
            tracing::info!(topic = ?topic.renamed(), hash = ?op.hash().renamed(), "publishing mailbox operation");
            store
                .entry(topic)
                .or_default()
                .entry(author)
                .or_default()
                .insert(seq_num, op);
        }
        Ok(())
    }

    async fn fetch(
        &self,
        topic: LogId,
        min_heights: &[(Op::Author, u64)],
    ) -> anyhow::Result<FetchResponse<Op>> {
        let ops = self.mailbox.ops.read().await;
        if let Some(ops) = ops.get(&topic) {
            let mut new = vec![];
            let mut missing = HashMap::new();
            for (author, height) in min_heights {
                let mut gaps = vec![];
                if let Some(slots) = ops.get(author) {
                    let last_seq = slots.last_key_value().map(|(seq, _)| *seq).unwrap_or(0);
                    let max = (*height).max(last_seq);
                    for i in 0..=max {
                        if let Some(op) = slots.get(&i) {
                            if i > *height {
                                new.push(op.clone());
                            }
                        } else {
                            gaps.push(i);
                        }
                    }
                    missing.insert(*author, gaps);
                }
            }

            tracing::info!(
                topic = ?topic.renamed(),
                num = new.len(),
                hashes = ?new.iter().map(|op: &Op| op.hash().renamed()).collect::<Vec<_>>(),
                "fetching mailbox operations"
            );

            Ok(FetchResponse { ops: new, missing })
        } else {
            Ok(FetchResponse {
                ops: vec![],
                missing: HashMap::new(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::Topic;

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Msg(u64);

    impl MailboxItem for Msg {
        type Hash = u64;
        type Author = ();

        fn hash(&self) -> Self::Hash {
            self.0
        }

        fn author(&self) -> Self::Author {
            ()
        }

        fn seq_num(&self) -> u64 {
            self.0
        }
    }

    fn m(s: u64) -> Msg {
        Msg(s)
    }

    #[tokio::test]
    async fn test_mem_mailbox() {
        let mailbox = MemMailbox::<Msg>::new();
        let client = mailbox.client();

        let tt: [LogId; 2] = [Topic::random().into(), Topic::random().into()];

        assert!(client.fetch(tt[0], &[]).await.unwrap().ops.is_empty());
        assert!(client.fetch(tt[1], &[]).await.unwrap().ops.is_empty());

        client
            .publish(tt[0], vec![m(0), m(1), m(2), m(3)])
            .await
            .unwrap();

        assert_eq!(
            client.fetch(tt[0], &[]).await.unwrap().ops,
            vec![m(0), m(1)]
        );
        assert_eq!(
            client.fetch(tt[1], &[]).await.unwrap().ops,
            vec![m(2), m(3)]
        );
        assert!(client.fetch(tt[0], &[]).await.unwrap().ops.is_empty());
        assert!(client.fetch(tt[1], &[]).await.unwrap().ops.is_empty());

        client.publish(tt[0], vec![m(4), m(5)]).await.unwrap();
        client.publish(tt[1], vec![m(6), m(7)]).await.unwrap();

        assert_eq!(
            client.fetch(tt[0], &[]).await.unwrap().ops,
            vec![m(4), m(5)]
        );
        assert_eq!(
            client.fetch(tt[1], &[]).await.unwrap().ops,
            vec![m(6), m(7)]
        );
        assert!(client.fetch(tt[0], &[]).await.unwrap().ops.is_empty());
        assert!(client.fetch(tt[1], &[]).await.unwrap().ops.is_empty());
    }

    #[tokio::test]
    async fn test_mem_mailbox_gaps() {
        todo!()
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
