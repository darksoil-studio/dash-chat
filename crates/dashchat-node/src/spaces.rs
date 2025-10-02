mod control_message;
mod store;

pub use control_message::*;
pub use store::*;

use p2panda_spaces::manager::Manager;
use p2panda_spaces::traits::SpaceId;
use p2panda_spaces::types::StrongRemoveResolver;

use crate::chat::ChatId;
use crate::forge::DashForge;

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
