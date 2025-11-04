//! Types to use with p2panda-spaces

mod control_message;
mod forge;

pub use control_message::*;

use p2panda_spaces::manager::Manager;
use p2panda_spaces::test_utils::TestKeyStore;
use p2panda_spaces::types::StrongRemoveResolver;

use crate::chat::BaseId;
use crate::stores::SpacesStore;
pub use forge::DashForge;

pub type TestConditions = ();

pub type DashSpace = p2panda_spaces::space::Space<
    BaseId,
    SpacesStore,
    TestKeyStore,
    DashForge,
    SpaceControlMessage,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;

pub type DashManager = Manager<
    BaseId,
    SpacesStore,
    TestKeyStore,
    DashForge,
    SpaceControlMessage,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;
