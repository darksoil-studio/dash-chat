use std::{str::FromStr, sync::Arc};

use p2panda_auth::traits::Conditions;
use p2panda_core::{
    PrivateKey,
    cbor::{decode_cbor, encode_cbor},
};
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
    member::Member,
    space::SpaceState,
    store::{AuthStore, KeyStore, MessageStore, SpaceStore},
    types::AuthGroupState,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::*;

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

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
#[serde(into = "String", try_from = "String")]
pub struct MemberCode(LongTermKeyBundle, ActorId);

impl From<Member> for MemberCode {
    fn from(member: Member) -> Self {
        Self(member.key_bundle().clone(), member.id())
    }
}

impl From<MemberCode> for Member {
    fn from(member_code: MemberCode) -> Self {
        Member::new(member_code.1, member_code.0)
    }
}

impl std::fmt::Display for MemberCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = encode_cbor(&(self.0.clone(), self.1)).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", hex::encode(bytes))
    }
}

impl FromStr for MemberCode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let (long_term_key_bundle, actor_id) = decode_cbor(bytes.as_slice())?;
        Ok(Self(long_term_key_bundle, actor_id))
    }
}

impl From<MemberCode> for String {
    fn from(code: MemberCode) -> Self {
        code.to_string()
    }
}

impl TryFrom<String> for MemberCode {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(MemberCode::from_str(&value).unwrap())
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
