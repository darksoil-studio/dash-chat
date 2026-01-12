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
                    if !gaps.is_empty() {
                        missing.insert(*author, gaps);
                    }
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
    use pretty_assertions::assert_eq;

    use crate::Topic;

    use super::*;

    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize, derive_more::Debug)]
    #[debug("Msg({author} {seq})")]
    struct Msg {
        author: char,
        seq: u64,
    }

    impl MailboxItem for Msg {
        type Author = char;
        type Hash = (Self::Author, u64);

        fn hash(&self) -> Self::Hash {
            (self.author, self.seq)
        }

        fn author(&self) -> Self::Author {
            self.author.clone()
        }

        fn seq_num(&self) -> u64 {
            self.seq
        }
    }

    fn m(author: char, seq: u64) -> Msg {
        Msg { author, seq }
    }

    fn mm(author: char, r: std::ops::Range<u64>) -> Vec<Msg> {
        r.map(|i| m(author, i)).collect()
    }

    #[tokio::test]
    async fn test_mem_mailbox() {
        let mailbox = MemMailbox::<Msg>::new();
        let client = mailbox.client();

        let a = '.';
        let tt: [LogId; 3] = [
            Topic::random().into(),
            Topic::random().into(),
            Topic::random().into(),
        ];

        let empty = FetchResponse {
            ops: vec![],
            missing: HashMap::new(),
        };
        assert_eq!(client.fetch(tt[0], &[]).await.unwrap(), empty);
        assert_eq!(client.fetch(tt[1], &[]).await.unwrap(), empty);
        assert_eq!(client.fetch(tt[2], &[]).await.unwrap(), empty);

        // only the first half
        client.publish(tt[0], mm(a, 0..2)).await.unwrap();
        // only the last half
        client.publish(tt[1], mm(a, 2..4)).await.unwrap();
        // both halves
        client.publish(tt[2], mm(a, 0..4)).await.unwrap();

        assert_eq!(
            client.fetch(tt[0], &[(a, 3)]).await.unwrap(),
            FetchResponse {
                ops: vec![],
                missing: HashMap::from([(a, vec![2, 3])]),
            }
        );
        assert_eq!(
            client.fetch(tt[1], &[(a, 3)]).await.unwrap(),
            FetchResponse {
                ops: vec![],
                missing: HashMap::from([(a, vec![0, 1])]),
            }
        );
        assert_eq!(client.fetch(tt[2], &[(a, 3)]).await.unwrap(), empty);

        assert_eq!(
            client.fetch(tt[0], &[(a, 4)]).await.unwrap(),
            FetchResponse {
                ops: vec![],
                missing: HashMap::from([(a, vec![2, 3, 4])]),
            }
        );
        assert_eq!(
            client.fetch(tt[1], &[(a, 4)]).await.unwrap(),
            FetchResponse {
                ops: vec![],
                missing: HashMap::from([(a, vec![0, 1, 4])]),
            }
        );
        assert_eq!(
            client.fetch(tt[2], &[(a, 4)]).await.unwrap(),
            FetchResponse {
                ops: vec![],
                missing: HashMap::from([(a, vec![4])]),
            }
        );

        client.publish(tt[0], vec![m(a, 4)]).await.unwrap();
        client.publish(tt[1], vec![m(a, 4)]).await.unwrap();
        client.publish(tt[2], vec![m(a, 4)]).await.unwrap();
        assert_eq!(
            client.fetch(tt[0], &[(a, 3)]).await.unwrap(),
            FetchResponse {
                ops: vec![m(a, 4)],
                missing: HashMap::from([(a, vec![2, 3])]),
            }
        );
        assert_eq!(
            client.fetch(tt[1], &[(a, 3)]).await.unwrap(),
            FetchResponse {
                ops: vec![m(a, 4)],
                missing: HashMap::from([(a, vec![0, 1])]),
            }
        );
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
