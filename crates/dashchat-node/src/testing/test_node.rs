use std::{
    collections::{BTreeSet, HashMap, HashSet},
    time::{Duration, Instant},
};

use tokio::sync::mpsc::Receiver;

use crate::{
    NodeConfig, Notification, ShortId,
    node::{Node, NodeLocalData},
    testing::{AliasedId, introduce},
    topic::Topic,
};

#[derive(Clone, derive_more::Deref, derive_more::Debug)]
#[debug("TestNode({})", self.0.public_key().alias())]
pub struct TestNode(Node);

impl TestNode {
    pub async fn new(config: NodeConfig, alias: Option<&str>) -> (Self, Watcher<Notification>) {
        let local_data = NodeLocalData::new_random();
        let (notification_tx, notification_rx) = tokio::sync::mpsc::channel(100);
        if let Some(alias) = alias {
            local_data.private_key.public_key().aliased(alias);
        }
        let node = Self(
            Node::new(local_data, config, Some(notification_tx))
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
    pub async fn new(node_config: NodeConfig, config: ClusterConfig, aliases: [&str; N]) -> Self {
        let nodes = futures::future::join_all(
            (0..N).map(|i| TestNode::new(node_config.clone(), Some(aliases[i]))),
        )
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
        // TODO: Fix this when we have a proper way to access operations
        // The operations field is now private in the new p2panda-store version
        let sets = nodes
            .iter()
            .map(|node| {
                let ops = node.op_store.processed_ops.read().unwrap();

                topics
                    .iter()
                    .flat_map(|topic| {
                        ops.get(topic)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|h| format!("{} {}", h.short(), h.alias()))
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
    tracing::info!("=== awaiting consistency for up to {:?} ===", timeout);
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
