//! Types to use with p2panda-spaces

mod control_message;
mod forge;

pub use control_message::*;

use p2panda_spaces::manager::Manager;
use p2panda_spaces::traits::SpaceId;
use p2panda_spaces::types::StrongRemoveResolver;

use crate::chat::ChatId;
use crate::stores::SpacesStore;
pub use forge::DashForge;

pub type TestConditions = ();

impl SpaceId for ChatId {}

pub type DashSpace = p2panda_spaces::space::Space<
    ChatId,
    SpacesStore,
    DashForge,
    SpaceControlMessage,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;

pub type DashManager = Manager<
    ChatId,
    SpacesStore,
    DashForge,
    SpaceControlMessage,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;
