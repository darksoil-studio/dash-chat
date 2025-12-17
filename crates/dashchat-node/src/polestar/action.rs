use serde::{Deserialize, Serialize};

use crate::topic::LogId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Space(SpaceAction),
    AuthorOp {
        log_id: LogId,
        hash: p2panda_core::Hash,
    },
    ProcessOp,
    BufferOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpaceAction {}
