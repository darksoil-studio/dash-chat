//! Types to use with p2panda-spaces

mod control_message;
mod forge;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::Context;
pub use control_message::*;

use named_id::{Nameable, Rename};
use p2panda_auth::Access;
use p2panda_core::PrivateKey;
use p2panda_encryption::Rng;
use p2panda_encryption::crypto::x25519::SecretKey;
use p2panda_spaces::manager::Manager;
use p2panda_spaces::test_utils::TestKeyStore;
use p2panda_spaces::types::StrongRemoveResolver;
use p2panda_spaces::{ActorId, Event, Member};

use crate::Topic;
use crate::chat::ChatId;
use crate::stores::{OpStore, SpacesStore};
pub use forge::DashForge;

pub type TestConditions = ();

static CREATED_SPACES: LazyLock<Mutex<HashSet<ChatId>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub type DashGroup = p2panda_spaces::group::Group<
    ChatId,
    SpacesStore,
    TestKeyStore,
    DashForge,
    SpaceOperation,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;

pub type DashSpace = p2panda_spaces::space::Space<
    ChatId,
    SpacesStore,
    TestKeyStore,
    DashForge,
    SpaceOperation,
    TestConditions,
    StrongRemoveResolver<TestConditions>,
>;

#[derive(Clone, derive_more::Deref)]
pub struct DashManager {
    #[deref]
    manager: Manager<
        ChatId,
        SpacesStore,
        TestKeyStore,
        DashForge,
        SpaceOperation,
        TestConditions,
        StrongRemoveResolver<TestConditions>,
    >,
    pub store: SpacesStore,
}

impl DashManager {
    pub async fn new(
        private_key: PrivateKey,
        chat_actor_id: ActorId,
        spaces_store: SpacesStore,
        op_store: OpStore,
        rng: Rng,
    ) -> anyhow::Result<(Self, Vec<SpaceOperation>)> {
        let credentials = p2panda_spaces::Credentials::from_keys(
            private_key.clone(),
            SecretKey::from_rng(&rng).context("Failed to generate secret key")?,
        );

        let forge = DashForge {
            private_key,
            chat_actor_id,
            store: spaces_store.clone(),
            op_store: op_store,
        };

        let key_store = p2panda_spaces::test_utils::TestKeyStore::new();
        let manager =
            Manager::new(spaces_store.clone(), key_store, forge, credentials, rng).await?;
        let device_group_msgs = {
            let (group, msgs, _event) = Box::pin(
                manager.create_group_with_id(&[(manager.id(), Access::manage())], chat_actor_id),
            )
            .await?;

            crate::testing::alias_space_messages(
                "create_device_group",
                Topic::global(),
                msgs.iter(),
            );

            msgs
        };

        Ok((
            Self {
                manager,
                store: spaces_store,
            },
            device_group_msgs,
        ))
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
    pub async fn create_space(
        &self,
        topic: impl Into<ChatId> + Nameable,
        initial_members: &[(ActorId, Access<TestConditions>)],
    ) -> anyhow::Result<(
        DashSpace,
        Vec<SpaceOperation>,
        Vec<Event<ChatId, TestConditions>>,
    )> {
        let topic = topic.into();
        let initial = initial_members
            .iter()
            .map(|(id, _)| id.renamed())
            .collect::<Vec<_>>();
        tracing::info!(topic = ?topic.renamed(), ?initial, "SM: creating space");

        {
            if !CREATED_SPACES.lock().unwrap().insert(topic) {
                panic!("Space already created: {}", topic.renamed());
            }
        }

        // XXX: this future often overflows the stack, so we box it to put it on the heap
        let manager = self.manager.clone();
        let (space, msgs, _event) = Box::pin(manager.create_space(topic, initial_members)).await?;
        Ok((space, msgs, _event))
    }
}
