use std::sync::Arc;

use p2panda_auth::traits::Conditions;
use p2panda_core::PrivateKey;
use p2panda_encryption::{
    key_bundle::LongTermKeyBundle,
    key_manager::{KeyManager, KeyManagerState},
    key_registry::KeyRegistryState,
};
use p2panda_spaces::{
    ActorId, OperationId,
    space::SpaceState,
    traits::{AuthStore, MessageStore, SpaceId, SpaceStore},
    types::AuthGroupState,
};
use tokio::sync::RwLock;

use crate::chat::ChatId;
use crate::spaces::{SpaceControlMessage, TestConditions};

// Conversion traits between ChatId and TestSpaceId
impl From<ChatId> for p2panda_spaces::test_utils::TestSpaceId {
    fn from(chat_id: ChatId) -> Self {
        // Convert the first 8 bytes of the ChatId to a usize
        let bytes = chat_id.0;
        let mut result = 0u64;
        for (i, &byte) in bytes.iter().enumerate().take(8) {
            result |= (byte as u64) << (i * 8);
        }
        result as usize
    }
}

impl From<p2panda_spaces::test_utils::TestSpaceId> for ChatId {
    fn from(test_id: p2panda_spaces::test_utils::TestSpaceId) -> Self {
        // Convert usize back to ChatId by padding with zeros
        let mut bytes = [0u8; 32];
        let id_bytes = test_id.to_le_bytes();
        bytes[..id_bytes.len()].copy_from_slice(&id_bytes);
        ChatId(bytes)
    }
}

pub type TestStore =
    p2panda_spaces::test_utils::store::MemoryStore<SpaceControlMessage, TestConditions>;

pub fn create_test_store(private_key: PrivateKey) -> TestStore {
    TestStore::new()
}

/////////////////////////////////////////////////////////

pub type SpacesStore = SharedSpaceStore<ChatIdSpaceStore<TestStore>>;

#[derive(Debug, derive_more::Deref)]
pub struct SharedSpaceStore<S>(Arc<RwLock<S>>);

impl SharedSpaceStore<ChatIdSpaceStore<TestStore>> {
    pub fn new(private_key: PrivateKey) -> Self {
        Self(Arc::new(RwLock::new(ChatIdSpaceStore::new(
            create_test_store(private_key),
        ))))
    }
}

impl<S> From<S> for SharedSpaceStore<S> {
    fn from(store: S) -> Self {
        Self(Arc::new(RwLock::new(store)))
    }
}

impl<S> Clone for SharedSpaceStore<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/////////////////////////////////////////////////////////////////

impl<S, ID, M, C> SpaceStore<ID, M, C> for SharedSpaceStore<S>
where
    ID: SpaceId + std::hash::Hash,
    M: Clone,
    C: Conditions,
    S: SpaceStore<ID, M, C>,
{
    type Error = S::Error;

    async fn space(&self, id: &ID) -> Result<Option<SpaceState<ID, M, C>>, Self::Error> {
        self.read().await.space(id).await
    }

    async fn has_space(&self, id: &ID) -> Result<bool, Self::Error> {
        self.read().await.has_space(id).await
    }

    async fn spaces_ids(&self) -> Result<Vec<ID>, Self::Error> {
        self.read().await.spaces_ids().await
    }

    async fn set_space(&self, id: &ID, y: SpaceState<ID, M, C>) -> Result<(), Self::Error> {
        self.write().await.set_space(id, y).await
    }
}

impl<S, C> AuthStore<C> for SharedSpaceStore<S>
where
    C: Conditions,
    S: AuthStore<C>,
{
    type Error = S::Error;

    async fn auth(&self) -> Result<AuthGroupState<C>, Self::Error> {
        self.read().await.auth().await
    }

    async fn set_auth(&self, y: &AuthGroupState<C>) -> Result<(), Self::Error> {
        self.write().await.set_auth(y).await
    }
}

impl<S, M> MessageStore<M> for SharedSpaceStore<S>
where
    M: Clone,
    S: MessageStore<M>,
{
    type Error = S::Error;

    async fn message(&self, id: &OperationId) -> Result<Option<M>, Self::Error> {
        self.read().await.message(id).await
    }

    async fn set_message(&self, id: &OperationId, message: &M) -> Result<(), Self::Error> {
        self.write().await.set_message(id, message).await
    }
}

// Wrapper to make TestStore work with ChatId instead of TestSpaceId
#[derive(Debug, Clone)]
pub struct ChatIdSpaceStore<S>(S);

impl<S> ChatIdSpaceStore<S> {
    pub fn new(store: S) -> Self {
        Self(store)
    }
}

impl<S> SpaceStore<ChatId, SpaceControlMessage, TestConditions> for ChatIdSpaceStore<S>
where
    S: SpaceStore<p2panda_spaces::test_utils::TestSpaceId, SpaceControlMessage, TestConditions>,
{
    type Error = S::Error;

    async fn space(
        &self,
        id: &ChatId,
    ) -> Result<Option<SpaceState<ChatId, SpaceControlMessage, TestConditions>>, Self::Error> {
        // Convert ChatId to TestSpaceId for the underlying store
        let test_space_id = p2panda_spaces::test_utils::TestSpaceId::from(*id);
        let result = self.0.space(&test_space_id).await?;

        // Convert the result back to use ChatId
        Ok(result.map(|state| SpaceState {
            space_id: *id,
            group_id: state.group_id,
            auth_y: state.auth_y,
            encryption_y: state.encryption_y,
        }))
    }

    async fn has_space(&self, id: &ChatId) -> Result<bool, Self::Error> {
        let test_space_id = p2panda_spaces::test_utils::TestSpaceId::from(*id);
        self.0.has_space(&test_space_id).await
    }

    async fn spaces_ids(&self) -> Result<Vec<ChatId>, Self::Error> {
        let test_ids = self.0.spaces_ids().await?;
        Ok(test_ids.into_iter().map(|id| id.into()).collect())
    }

    async fn set_space(
        &self,
        id: &ChatId,
        y: SpaceState<ChatId, SpaceControlMessage, TestConditions>,
    ) -> Result<(), Self::Error> {
        let test_space_id = p2panda_spaces::test_utils::TestSpaceId::from(*id);
        let test_state = SpaceState {
            space_id: test_space_id,
            group_id: y.group_id,
            auth_y: y.auth_y,
            encryption_y: y.encryption_y,
        };
        self.0.set_space(&test_space_id, test_state).await
    }
}

impl<S> AuthStore<TestConditions> for ChatIdSpaceStore<S>
where
    S: AuthStore<TestConditions>,
{
    type Error = S::Error;

    async fn auth(&self) -> Result<AuthGroupState<TestConditions>, Self::Error> {
        self.0.auth().await
    }

    async fn set_auth(&self, y: &AuthGroupState<TestConditions>) -> Result<(), Self::Error> {
        self.0.set_auth(y).await
    }
}

impl<S> MessageStore<SpaceControlMessage> for ChatIdSpaceStore<S>
where
    S: MessageStore<SpaceControlMessage>,
{
    type Error = S::Error;

    async fn message(&self, id: &OperationId) -> Result<Option<SpaceControlMessage>, Self::Error> {
        self.0.message(id).await
    }

    async fn set_message(
        &self,
        id: &OperationId,
        message: &SpaceControlMessage,
    ) -> Result<(), Self::Error> {
        self.0.set_message(id, message).await
    }
}
