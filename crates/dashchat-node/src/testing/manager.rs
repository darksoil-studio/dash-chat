use crate::spaces::{DashForge, DashManager};
use crate::stores::SpacesStore;
use p2panda_core::PrivateKey;
use p2panda_encryption::Rng;
use p2panda_encryption::crypto::x25519::SecretKey;

pub async fn test_manager() -> DashManager {
    let rng = Rng::default();

    let private_key = PrivateKey::new();

    let credentials = p2panda_spaces::Credentials::from_keys(
        private_key.clone(),
        SecretKey::from_rng(&rng).unwrap(),
    );
    let spaces_store = SpacesStore::new();

    let forge = DashForge {
        public_key: private_key.public_key(),
        store: spaces_store.clone(),
    };

    let key_store = p2panda_spaces::test_utils::TestKeyStore::new();
    DashManager::new(spaces_store.clone(), key_store, forge, credentials, rng)
        .await
        .unwrap()
}
