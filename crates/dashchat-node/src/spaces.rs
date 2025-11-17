//! Types to use with p2panda-spaces

mod control_message;
mod forge;

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

use anyhow::Context;
pub use control_message::*;

use p2panda_auth::Access;
use p2panda_core::PrivateKey;
use p2panda_encryption::Rng;
use p2panda_encryption::crypto::x25519::SecretKey;
use p2panda_spaces::manager::Manager;
use p2panda_spaces::test_utils::TestKeyStore;
use p2panda_spaces::types::StrongRemoveResolver;
use p2panda_spaces::{ActorId, Event, Member};

use crate::chat::ChatId;
use crate::stores::SpacesStore;
use crate::testing::AliasedId;
pub use forge::DashForge;

pub type TestConditions = ();

static CREATED_SPACES: LazyLock<Mutex<HashSet<ChatId>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub type DashSpace = p2panda_spaces::space::Space<
    ChatId,
    SpacesStore,
    TestKeyStore,
    DashForge,
    SpaceControlMessage,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;

#[derive(Clone, derive_more::Deref)]
pub struct DashManager {
    manager: Manager<
        ChatId,
        SpacesStore,
        TestKeyStore,
        DashForge,
        SpaceControlMessage,
        TestConditions,
        StrongRemoveResolver<TestConditions>,
    >,
}

impl DashManager {
    pub async fn new(
        private_key: PrivateKey,
        spaces_store: SpacesStore,
        rng: Rng,
    ) -> anyhow::Result<Self> {
        let credentials = p2panda_spaces::Credentials::from_keys(
            private_key.clone(),
            SecretKey::from_rng(&rng).context("Failed to generate secret key")?,
        );

        let forge = DashForge {
            public_key: private_key.public_key(),
            store: spaces_store.clone(),
        };

        let key_store = p2panda_spaces::test_utils::TestKeyStore::new();
        let manager =
            Manager::new(spaces_store.clone(), key_store, forge, credentials, rng).await?;

        Ok(Self { manager })
    }

    pub async fn create_space(
        &self,
        topic: impl Into<ChatId> + AliasedId,
        initial_members: &[(ActorId, Access<TestConditions>)],
    ) -> anyhow::Result<(
        DashSpace,
        Vec<SpaceControlMessage>,
        Vec<Event<ChatId, TestConditions>>,
    )> {
        let initial = initial_members
            .iter()
            .map(|(id, _)| id.alias())
            .collect::<Vec<_>>();
        tracing::info!(topic = topic.alias(), ?initial, "SM: creating space");
        let topic = topic.into();

        {
            if !CREATED_SPACES.lock().unwrap().insert(topic) {
                panic!("Space already created: {}", topic.alias());
            }
        }

        println!("STACK OVERFLOW HERE");
        let (space, msgs, _event) = self.manager.create_space(topic, initial_members).await?;
        println!("CAN'T SEE THIS CUZ STACK OVERFLOWED");
        Ok((space, msgs, _event))
    }
}
