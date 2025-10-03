use std::sync::Arc;

use p2panda_auth::traits::Conditions;
use p2panda_core::PrivateKey;
use p2panda_encryption::{
    Rng,
    crypto::x25519::SecretKey,
    key_bundle::{Lifetime, LongTermKeyBundle},
    key_manager::{KeyManager, KeyManagerState},
    key_registry::KeyRegistryState,
    traits::PreKeyManager,
};
use p2panda_spaces::{
    ActorId, OperationId,
    auth::orderer::AuthOrderer,
    space::SpaceState,
    store::{AuthStore, KeyStore, MessageStore, SpaceStore},
    traits::SpaceId,
    types::AuthGroupState,
};
use tokio::sync::RwLock;

use crate::{
    ChatId,
    spaces::{SpaceControlMessage, TestConditions},
};

pub type TestStore =
    p2panda_spaces::test_utils::MemoryStore<ChatId, SpaceControlMessage, TestConditions>;

pub fn create_test_store(private_key: PrivateKey) -> TestStore {
    let rng = Rng::default();

    let my_id: ActorId = private_key.public_key().into();

    let key_manager_y = {
        let identity_secret = SecretKey::from_bytes(rng.random_array().unwrap());
        KeyManager::init(&identity_secret, Lifetime::default(), &rng).unwrap()
    };

    let orderer_y = AuthOrderer::init();
    let auth_y = AuthGroupState::new(orderer_y);
    let store = TestStore::new(my_id, key_manager_y, auth_y);

    store
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

impl<S: KeyStore> SharedSpaceStore<S> {
    pub async fn long_term_key_bundle(&self) -> Result<LongTermKeyBundle, S::Error> {
        let store = self.read().await;
        let y = store.key_manager().await?;
        Ok(KeyManager::prekey_bundle(&y))
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

    async fn set_space(&mut self, id: &ID, y: SpaceState<ID, M, C>) -> Result<(), Self::Error> {
        self.write().await.set_space(id, y).await
    }
}

impl<S> KeyStore for SharedSpaceStore<S>
where
    S: KeyStore,
{
    type Error = S::Error;

    async fn key_manager(&self) -> Result<KeyManagerState, Self::Error> {
        self.read().await.key_manager().await
    }

    async fn key_registry(&self) -> Result<KeyRegistryState<ActorId>, Self::Error> {
        self.read().await.key_registry().await
    }

    async fn set_key_manager(&mut self, y: &KeyManagerState) -> Result<(), Self::Error> {
        self.write().await.set_key_manager(y).await
    }

    async fn set_key_registry(&mut self, y: &KeyRegistryState<ActorId>) -> Result<(), Self::Error> {
        self.write().await.set_key_registry(y).await
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

    async fn set_auth(&mut self, y: &AuthGroupState<C>) -> Result<(), Self::Error> {
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

    async fn set_message(&mut self, id: &OperationId, message: &M) -> Result<(), Self::Error> {
        self.write().await.set_message(id, message).await
    }
}
