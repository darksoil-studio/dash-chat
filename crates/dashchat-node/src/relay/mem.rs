use super::*;

use std::{collections::HashMap, sync::Arc};

use p2panda_core::Body;
use tokio::sync::{Mutex, RwLock};

use crate::{Header, topic::LogId};

impl From<Operation> for RelayOperation {
    fn from(op: Operation) -> Self {
        Self {
            header: op.header,
            body: op.body,
        }
    }
}

impl From<RelayOperation> for Operation {
    fn from(op: RelayOperation) -> Self {
        Self {
            hash: op.header.hash(),
            header: op.header,
            body: op.body,
        }
    }
}

impl From<(Header, Option<Body>)> for RelayOperation {
    fn from((header, body): (Header, Option<Body>)) -> Self {
        Self { header, body }
    }
}

/// A client for the memory relay.
/// This client is stateful, so all requests for a node should go through a single client
/// instance. State is shared between all cloned copies of this.
#[derive(Clone)]
pub struct MemRelayClient<Op: RelayBlob = RelayOperation> {
    relay: MemRelay<Op>,
    latest: Arc<Mutex<HashMap<LogId, usize>>>,
}

#[derive(Clone)]
pub struct MemRelay<Op: RelayBlob = RelayOperation> {
    ops: Arc<RwLock<HashMap<LogId, Vec<Op>>>>,
}

impl<Op: RelayBlob> MemRelay<Op> {
    pub fn new() -> Self {
        Self {
            ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn client(&self) -> MemRelayClient<Op> {
        MemRelayClient {
            relay: self.clone(),
            latest: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl<Op: RelayBlob> RelayClient<Op> for MemRelayClient<Op> {
    async fn publish(&self, topic: LogId, op: Op) -> Result<(), anyhow::Error> {
        let mut ops = self.relay.ops.write().await;
        tracing::info!(?topic, "publishing relay operation");
        ops.entry(topic).or_insert_with(Vec::new).push(op.into());
        Ok(())
    }

    async fn fetch(&self, topic: LogId) -> anyhow::Result<Vec<Op>> {
        let ops = self.relay.ops.read().await;
        tracing::info!(?topic, num = ops.len(), "fetching relay operations");
        let mut since_lock = self.latest.lock().await;
        let since = *since_lock.get(&topic).unwrap_or(&0);
        if let Some(ops) = ops.get(&topic) {
            since_lock.insert(topic, ops.len());
            Ok(ops.iter().skip(since).cloned().collect())
        } else {
            Ok(vec![])
        }
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
    async fn test_mem_relay() {
        let relay = MemRelay::<Bytes>::new();
        let client = relay.client();

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
    // async fn test_mem_relay_clients() {
    //     let relay = MemRelay::new();

    //     let cc = [relay.client(), relay.client()];

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
    //     relay.publish(t1.into(), op).await.unwrap();
    //     let ops = relay.fetch(topic, 0).await.unwrap();
    //     assert_eq!(ops.len(), 1);
    //     assert_eq!(ops[0], op);
    // }
}
