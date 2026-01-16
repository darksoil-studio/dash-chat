use serde::{Deserialize, Serialize};

use crate::topic::TopicId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Space(SpaceAction),
    AuthorOp {
        topic: TopicId,
        hash: p2panda_core::Hash,
    },
    ProcessOp,
    BufferOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpaceAction {}
