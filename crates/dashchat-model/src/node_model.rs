use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
    marker::PhantomData,
};

use anyhow::{anyhow, bail};
use exhaustive::Exhaustive;
use polestar::{ext::MapExt, id::Id, prelude::*};
use serde::{Deserialize, Serialize};

use crate::op_model as op;

/*                   █████     ███
                    ░░███     ░░░
  ██████    ██████  ███████   ████   ██████  ████████
 ░░░░░███  ███░░███░░░███░   ░░███  ███░░███░░███░░███
  ███████ ░███ ░░░   ░███     ░███ ░███ ░███ ░███ ░███
 ███░░███ ░███  ███  ░███ ███ ░███ ░███ ░███ ░███ ░███
░░████████░░██████   ░░█████  █████░░██████  ████ █████
 ░░░░░░░░  ░░░░░░     ░░░░░  ░░░░░  ░░░░░░  ░░░░ ░░░░░   */

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Exhaustive, Serialize, Deserialize, derive_more::Display,
)]
pub enum Action<Hash: Id, Pubkey: Id, Topic: Id> {
    Op(OpAction<Hash, Pubkey, Topic>),
    Topic(TopicAction<Topic>),
}

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Exhaustive, Serialize, Deserialize, derive_more::Display,
)]
pub enum OpAction<Hash: Id, Pubkey: Id, Topic: Id> {
    #[display("CreateOp(T:{topic} H:{hash})")]
    CreateOp { topic: Topic, hash: Hash },
    #[display("ReceiveOp(T:{topic} H:{hash} <- {from})")]
    ReceiveOp {
        topic: Topic,
        hash: Hash,
        from: Pubkey,
    },
    #[display("IngestOp(H:{hash})")]
    IngestOp { hash: Hash },
    #[display("ProcessOp(H:{hash})")]
    ProcessOp { hash: Hash },
    #[display("BufferOp(H:{hash} Deps:{deps:?})")]
    BufferOp { hash: Hash, deps: BTreeSet<Hash> },
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Exhaustive,
    Serialize,
    Deserialize,
    derive_more::Display,
)]
pub enum TopicAction<Topic: Id> {
    Subscribe { topic: Topic },
}

/*                                  █████       ███
                                   ░░███       ░░░
 █████████████    ██████    ██████  ░███████   ████  ████████    ██████
░░███░░███░░███  ░░░░░███  ███░░███ ░███░░███ ░░███ ░░███░░███  ███░░███
 ░███ ░███ ░███   ███████ ░███ ░░░  ░███ ░███  ░███  ░███ ░███ ░███████
 ░███ ░███ ░███  ███░░███ ░███  ███ ░███ ░███  ░███  ░███ ░███ ░███░░░
 █████░███ █████░░████████░░██████  ████ █████ █████ ████ █████░░██████
░░░░░ ░░░ ░░░░░  ░░░░░░░░  ░░░░░░  ░░░░ ░░░░░ ░░░░░ ░░░░ ░░░░░  ░░░░░░  */

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Model<Hash, Pubkey, Topic> {
    op: op::Model,
    _phantom: PhantomData<(Hash, Pubkey, Topic)>,
}

impl<Hash: Id, Pubkey: Id, Topic: Id> Machine for Model<Hash, Pubkey, Topic> {
    type State = State<Hash, Topic>;
    type Action = Action<Hash, Pubkey, Topic>;
    type Fx = ();
    type Error = anyhow::Error;

    fn transition(&self, mut state: Self::State, action: Self::Action) -> TransitionResult<Self> {
        use Action as A;
        use State as S;

        match action {
            A::Topic(topic_action) => match topic_action {
                TopicAction::Subscribe { topic } => {
                    if state.subs.insert(topic) {
                        bail!("Topic already subscribed: {:?}", topic);
                    }
                }
            },
            A::Op(op_action) => match op_action {
                OpAction::CreateOp { hash, topic } | OpAction::ReceiveOp { hash, topic, .. } => {
                    if !state.subs.contains(&topic) {
                        bail!("Topic not subscribed: {:?}", topic);
                    }
                    state.ops.insert(hash, self.op.initial().into());
                }
                OpAction::IngestOp { hash } => {
                    state.ops.owned_update(hash, |_, o| match o {
                        Phase::Op(o) => {
                            Ok((self.op.transition_(o, op::Action::Ingest)?.into(), ()))
                        }
                        Phase::Buffered { .. } => {
                            bail!("TODO")
                        }
                    })?;
                }
                OpAction::ProcessOp { hash } => {
                    state.ops.owned_update(hash, |_, o| match o {
                        Phase::Op(o) => {
                            Ok((self.op.transition_(o, op::Action::Process)?.into(), ()))
                        }
                        Phase::Buffered { hash, .. } => {
                            bail!("Operation is buffered: {:?}", hash);
                        }
                    })?;
                }
                OpAction::BufferOp { hash, deps } => {
                    bail!("TODO")
                }
            },
        }

        Ok((state, ()))
    }

    fn is_terminal(&self, s: &Self::State) -> bool {
        false
    }
}

impl<Hash: Id, Pubkey: Id, Topic: Id> Model<Hash, Pubkey, Topic> {
    pub fn new() -> Self {
        Self {
            op: op::Model::new(),
            _phantom: PhantomData,
        }
    }

    pub fn initial(&self) -> State<Hash, Topic> {
        State {
            ops: BTreeMap::new(),
            buffered: BTreeSet::new(),
            subs: BTreeSet::new(),
        }
    }
}

/*        █████               █████
         ░░███               ░░███
  █████  ███████    ██████   ███████    ██████
 ███░░  ░░░███░    ░░░░░███ ░░░███░    ███░░███
░░█████   ░███      ███████   ░███    ░███████
 ░░░░███  ░███ ███ ███░░███   ░███ ███░███░░░
 ██████   ░░█████ ░░████████  ░░█████ ░░██████
░░░░░░     ░░░░░   ░░░░░░░░    ░░░░░   ░░░░░░  */

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, derive_more::From, derive_more::Display)]
#[display("ops={:?}, buffered={:?}, subs={:?}", ops, buffered, subs)]
pub struct State<Hash: Id, Topic: Id> {
    pub ops: BTreeMap<Hash, Phase<Hash>>,
    pub buffered: BTreeSet<Hash>,
    pub subs: BTreeSet<Topic>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, derive_more::From)]
pub enum Phase<Hash: Id> {
    Op(op::State),
    Buffered { hash: Hash, deps: BTreeSet<Hash> },
}

/*█████                      █████
 ░░███                      ░░███
 ███████    ██████   █████  ███████    █████
░░░███░    ███░░███ ███░░  ░░░███░    ███░░
  ░███    ░███████ ░░█████   ░███    ░░█████
  ░███ ███░███░░░   ░░░░███  ░███ ███ ░░░░███
  ░░█████ ░░██████  ██████   ░░█████  ██████
   ░░░░░   ░░░░░░  ░░░░░░     ░░░░░  ░░░░░░*/

#[cfg(test)]
mod tests {
    use super::*;

    use polestar::diagram::write_dot;

    const AGENTS: usize = 2;
    const TOPICS: usize = 2;
    const OPS: usize = 2;

    type Key = UpTo<AGENTS>;
    type Topic = UpTo<TOPICS>;
    type Hash = UpTo<OPS>;

    #[test]
    fn test_todo() {
        tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();

        let model = Model::<Hash, Key, Topic>::new();
        let initial = model.initial();

        let traversal = model
            .traverse([initial])
            .ignore_loopbacks(true)
            .trace_every(1_000);

        // graph
        {
            let graph = traversal.diagram().unwrap();
            // let graph = graph.map(|_, n| n, |_, (i, e)| format!("n{i}: {e}"));
            write_dot("out.dot", &graph, &[]);
            println!(
                "wrote out.dot. nodes={}, edges={}",
                graph.node_count(),
                graph.edge_count()
            );
        }
    }
}
