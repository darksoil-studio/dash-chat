use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use named_id::*;
use p2panda_auth::Access;
use p2panda_spaces::ActorId;
use tokio::sync::{Mutex, mpsc::Receiver};

use crate::{
    ChatId, DeviceGroupPayload, NodeConfig, Notification, PK, Payload,
    node::{Node, NodeLocalData},
    spaces::DashGroup,
    testing::{behavior::Behavior, introduce},
    topic::{LogId, Topic},
};

#[derive(Clone, derive_more::Deref, derive_more::Debug)]
#[debug("TestNode({})", self.node.public_key().renamed())]
pub struct TestNode {
    #[deref]
    node: Node,
    pub watcher: Arc<Mutex<Watcher<Notification>>>,
}

impl TestNode {
    pub async fn new(config: NodeConfig, alias: Option<&str>) -> Self {
        let local_data = NodeLocalData::new_random();
        let (notification_tx, notification_rx) = tokio::sync::mpsc::channel(100);
        if let Some(alias) = alias {
            local_data.private_key.public_key().with_name(alias);
        }
        Self {
            node: Node::new(local_data, config, Some(notification_tx))
                .await
                .unwrap(),
            watcher: Arc::new(Mutex::new(Watcher(notification_rx))),
        }
    }

    pub fn behavior(&self) -> Behavior {
        Behavior::new(self.clone())
    }

    pub async fn get_groups(&self) -> anyhow::Result<Vec<ChatId>> {
        let groups = self.nodestate.chats.read().await.keys().cloned().collect();
        Ok(groups)
    }

    pub async fn get_members(
        &self,
        chat_id: ChatId,
    ) -> anyhow::Result<Vec<(p2panda_spaces::ActorId, Access)>> {
        Ok(self.space(chat_id).await?.members().await?)
    }

    pub async fn get_contacts(&self) -> anyhow::Result<Vec<ActorId>> {
        let group: DashGroup = self
            .manager
            .group(self.repped_group().group)
            .await?
            .ok_or_else(|| anyhow::anyhow!("device group not found"))?;
        let members = group
            .members()
            .await?
            .into_iter()
            .map(|(id, _)| PK::from(id).0)
            .collect::<Vec<_>>();

        let ids = self
            .get_interleaved_logs(self.device_group_topic().into(), members)
            .await?
            .into_iter()
            .filter_map(|(_, payload)| match payload {
                Some(Payload::DeviceGroup(DeviceGroupPayload::AddContact(qr))) => {
                    Some(qr.chat_actor_id)
                }
                _ => None,
            })
            .collect();
        Ok(ids)
    }

    pub async fn initialized_topics(&self) -> BTreeSet<LogId> {
        self.node
            .initialized_topics
            .read()
            .await
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
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
    nodes: [TestNode; N],
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
        let nodes = self.iter().map(|node| &node.network).collect::<Vec<_>>();
        introduce(nodes).await;
    }

    pub async fn nodes(&self) -> [TestNode; N] {
        self.nodes
            .iter()
            .map(|node| node.clone())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub async fn consistency(
        &self,
        log_ids: impl IntoIterator<Item = &LogId>,
    ) -> anyhow::Result<()> {
        consistency(self.nodes().await.iter(), log_ids, &self.config).await
    }
}

pub async fn consistency(
    nodes: impl IntoIterator<Item = &TestNode>,
    log_ids: impl IntoIterator<Item = &LogId>,
    config: &ClusterConfig,
) -> anyhow::Result<()> {
    let log_ids = log_ids.into_iter().collect::<HashSet<_>>();
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    wait_for_resetting(config.poll_interval, config.poll_timeout, || async {
        // TODO: Fix this when we have a proper way to access operations
        // The operations field is now private in the new p2panda-store version
        let sets = nodes
            .iter()
            .map(|node| {
                let ops = node.op_store.processed_ops.read().unwrap();

                log_ids
                    .iter()
                    .flat_map(|log_id| {
                        ops.get(log_id)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|h| format!("{} {}", h.short(), h.renamed()))
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
                n.op_store.report(log_ids.clone())
            );
        }
        println!("consistency report: {:#?}", diffs);
        anyhow::anyhow!("consistency check failed")
    })
}

#[derive(Debug, Clone, Default, PartialEq)]
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

impl<T: std::fmt::Debug> Watcher<T> {
    pub async fn watch_mapped<R>(
        &mut self,
        timeout: tokio::time::Duration,
        f: impl Fn(&T) -> Option<R>,
    ) -> anyhow::Result<R> {
        let timeout = tokio::time::sleep(timeout);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                item = self.0.recv() => {
                    match item {
                        Some(item) => match f(&item) {
                            Some(r) => return Ok(r),
                            None => continue,
                        },
                        None => return Err(anyhow::anyhow!("channel closed")),
                    }
                }
                _ = &mut timeout => return Err(anyhow::anyhow!("timeout")),
            }
        }
    }

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
                        Some(item) => if f(&item) {
                            return Ok(item)
                        } else {
                            continue
                        },
                        None => return Err(anyhow::anyhow!("channel closed")),
                    }
                }
                _ = &mut timeout => return Err(anyhow::anyhow!("timeout")),
            }
        }
    }
}

pub async fn wait_for<F, E>(poll: Duration, timeout: Duration, f: impl Fn() -> F) -> Result<(), E>
where
    F: Future<Output = Result<(), E>>,
{
    assert!(poll < timeout);
    let mut start = Instant::now();
    tracing::info!("=== wait_for() up to {:?} ===", timeout);
    loop {
        let result = f().await;
        match &result {
            Ok(()) => break,
            Err(_) => {
                if start.elapsed() > timeout {
                    return result;
                }

                tokio::time::sleep(poll).await;
            }
        }
    }
    tracing::info!("=== wait_for() success after {:?} ===", start.elapsed());
    Ok(())
}

pub async fn wait_for_resetting<F, E>(
    poll: Duration,
    timeout: Duration,
    f: impl Fn() -> F,
) -> Result<(), E>
where
    F: Future<Output = Result<(), E>>,
    E: std::fmt::Debug + PartialEq,
{
    assert!(poll < timeout);
    let mut start = Instant::now();
    tracing::info!("=== wait_for_resetting() up to {:?} ===", timeout);
    let mut previous = None;
    loop {
        let result = f().await;
        match &result {
            Ok(()) => break,
            Err(_) => {
                if start.elapsed() > timeout {
                    return result;
                }

                if previous.as_ref() != Some(&result) {
                    start = Instant::now();
                }

                previous = Some(result);

                tokio::time::sleep(poll).await;
            }
        }
    }
    tracing::info!(
        "=== wait_for_resetting() success after {:?} ===",
        start.elapsed()
    );
    Ok(())
}
