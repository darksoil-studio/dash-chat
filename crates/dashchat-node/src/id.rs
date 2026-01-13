use derive_more::{Deref, From};
use named_id::Rename;
use named_id::RenameAll;
use p2panda_core::PublicKey;
use p2panda_spaces::ActorId;
use serde::{Deserialize, Serialize};

/// The ID tied to a particular device.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    From,
    Deref,
    RenameAll,
)]
pub struct DeviceId(PublicKey);

/// The ID for an "agent" which may control multiple devices.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    From,
    Deref,
    RenameAll,
)]
pub struct AgentId(ActorId);
