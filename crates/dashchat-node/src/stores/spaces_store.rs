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

use crate::spaces::{SpaceControlMessage, TestConditions};

pub type TestStore =
    p2panda_spaces::test_utils::store::MemoryStore<SpaceControlMessage, TestConditions>;

pub fn create_test_store(private_key: PrivateKey) -> TestStore {
    TestStore::new()
}

/////////////////////////////////////////////////////////

pub type SpacesStore = SharedSpaceStore<TestStore>;

#[derive(Debug, derive_more::Deref)]
pub struct SharedSpaceStore<S>(Arc<RwLock<S>>);

impl SharedSpaceStore<TestStore> {
    pub fn new(private_key: PrivateKey) -> Self {
        Self(Arc::new(RwLock::new(create_test_store(private_key))))
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
