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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Exhaustive, Serialize, Deserialize)]
pub enum Action<H: Id, K: Id, T: Id> {
    Op(OpAction<H, K, T>),
    Topic(TopicAction<T>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Exhaustive, Serialize, Deserialize)]
pub enum OpAction<H: Id, K: Id, T: Id> {
    CreateOp { topic: T, hash: H },
    ReceiveOp { topic: T, hash: H, from: K },
    IngestOp { hash: H },
    ProcessOp { hash: H },
    BufferOp { hash: H, deps: BTreeSet<H> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Exhaustive, Serialize, Deserialize)]
pub enum TopicAction<T: Id> {
    Subscribe { topic: T },
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
pub struct Model<H, K, T> {
    op: op::Model,
    _phantom: PhantomData<(H, K, T)>,
}

impl<H: Id, K: Id, T: Id> Machine for Model<H, K, T> {
    type State = State<H, T>;
    type Action = Action<H, K, T>;
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
                            todo!()
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
                    todo!()
                }
            },
        }

        Ok((state, ()))
    }

    fn is_terminal(&self, s: &Self::State) -> bool {
        false
    }
}

impl<H: Id, K: Id, T: Id> Model<H, K, T> {
    pub fn new() -> Self {
        todo!()
    }

    pub fn initial(&self) -> State<H, T> {
        todo!()
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, derive_more::From)]
pub struct State<H: Id, T: Id> {
    pub ops: BTreeMap<H, Phase<H>>,
    pub buffered: BTreeSet<H>,
    pub subs: BTreeSet<T>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, derive_more::From)]
pub enum Phase<H: Id> {
    Op(op::State),
    Buffered { hash: H, deps: BTreeSet<H> },
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
}
