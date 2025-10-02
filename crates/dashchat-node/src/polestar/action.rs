use serde::{Deserialize, Serialize};

use crate::network::Topic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Space(SpaceAction),
    AuthorOp {
        topic: Topic,
        hash: p2panda_core::Hash,
    },
    ProcessOp,
    BufferOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpaceAction {}
