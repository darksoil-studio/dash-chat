use std::{
    collections::{BTreeSet, HashMap, HashSet},
    time::{Duration, Instant},
};

use p2panda_core::PrivateKey;
use tokio::sync::mpsc::Receiver;

use crate::{NodeConfig, Notification, ShortId, network::Topic, node::Node, testing::introduce};

#[derive(Debug, Clone, derive_more::Deref)]
pub struct TestNode(Node);

impl TestNode {
    pub async fn new() -> (Self, Watcher<Notification>) {
        let private_key = PrivateKey::new();
        let (notification_tx, notification_rx) = tokio::sync::mpsc::channel(100);
        let node = Self(
            Node::new(private_key, NodeConfig::default(), Some(notification_tx))
                .await
                .unwrap(),
        );
        (node, Watcher(notification_rx))
    }
}

#[derive(Clone, Debug)]
pub struct ClusterConfig {
    pub poll_interval: Duration,
    pub poll_timeout: Duration,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
            poll_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(derive_more::Deref)]
pub struct TestCluster<const N: usize> {
    #[deref]
    nodes: [(TestNode, Watcher<Notification>); N],
    pub config: ClusterConfig,
}

impl<const N: usize> TestCluster<N> {
    pub async fn new(config: ClusterConfig) -> Self {
        let nodes = futures::future::join_all((0..N).map(|_| TestNode::new()))
            .await
            .try_into()
            .unwrap_or_else(|_| panic!("expected {} nodes", N));
        Self { nodes, config }
    }

    pub async fn introduce_all(&self) {
        let nodes = self
            .iter()
            .map(|(node, _)| &node.0.network)
            .collect::<Vec<_>>();
        introduce(nodes).await;
    }

    pub async fn nodes(&self) -> [TestNode; N] {
        self.nodes
            .iter()
            .map(|(node, _)| node.clone())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub async fn consistency(
        &self,
        topics: impl IntoIterator<Item = &Topic>,
    ) -> anyhow::Result<()> {
        consistency(self.nodes().await.iter(), topics, &self.config).await
    }
}

pub async fn consistency(
    nodes: impl IntoIterator<Item = &TestNode>,
    topics: impl IntoIterator<Item = &Topic>,
    config: &ClusterConfig,
) -> anyhow::Result<()> {
    let topics = topics.into_iter().collect::<HashSet<_>>();
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    wait_for(config.poll_interval, config.poll_timeout, || async {
        let sets = nodes
            .iter()
            .map(|node| {
                node.op_store
                    .read_store()
                    .operations
                    .iter()
                    .filter_map(|(h, (t, _, _, _))| {
                        if topics.is_empty() || topics.contains(t) {
                            Some(h.short())
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>()
            })
            .collect::<Vec<_>>();
        let mut diffs = ConsistencyReport::new(sets);
        for i in 0..diffs.sets.len() {
            for j in 0..i {
                if i != j && diffs.sets[i] != diffs.sets[j] {
                    diffs.diffs.insert(
                        (i, j),
                        (diffs.sets[i].len() as isize - diffs.sets[j].len() as isize).abs(),
                    );
                }
            }
        }
        if diffs.diffs.is_empty() {
            Ok(())
        } else {
            Err(diffs)
        }
    })
    .await
    .map_err(|diffs| {
        for n in nodes {
            println!(
                ">>> {:?}\n{}\n",
                n.public_key(),
                n.op_store.report(topics.clone())
            );
        }
        println!("consistency report: {:#?}", diffs);
        anyhow::anyhow!("consistency check failed")
    })
}

#[derive(Debug, Clone, Default)]
pub struct ConsistencyReport {
    sets: Vec<BTreeSet<String>>,
    diffs: HashMap<(usize, usize), isize>,
}

impl ConsistencyReport {
    pub fn new(sets: Vec<BTreeSet<String>>) -> Self {
        Self {
            sets,
            diffs: HashMap::new(),
        }
    }
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Watcher<T>(Receiver<T>);

impl<T> Watcher<T> {
    pub async fn watch_for(
        &mut self,
        timeout: tokio::time::Duration,
        f: impl Fn(&T) -> bool,
    ) -> anyhow::Result<T> {
        let timeout = tokio::time::sleep(timeout);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                item = self.0.recv() => {
                    match item {
                        Some(item) if f(&item) => return Ok(item),
                        Some(_) => continue,
                        None => return Err(anyhow::anyhow!("channel closed")),
                    }
                }
                _ = &mut timeout => return Err(anyhow::anyhow!("timeout")),
            }
        }
    }
}

pub async fn wait_for<F, R>(poll: Duration, timeout: Duration, f: impl Fn() -> F) -> Result<(), R>
where
    F: Future<Output = Result<(), R>>,
    R: std::fmt::Debug,
{
    assert!(poll < timeout);
    let start = Instant::now();
    println!("===   wait for up to {:?}   ===", timeout);
    loop {
        match f().await {
            Ok(()) => break Ok(()),
            Err(r) => {
                if start.elapsed() > timeout {
                    return Err(r);
                }
                tokio::time::sleep(poll).await;
            }
        }
    }
}
