use crate::{
    Extensions,
    spaces::DashManager,
    stores::{OpStore, SpacesStore},
    topic::LogId,
};

use p2panda_core::PrivateKey;
use p2panda_encryption::Rng;

pub async fn test_manager() -> DashManager {
    let rng = Rng::default();
    let private_key = PrivateKey::new();

    let op_store = p2panda_store::MemoryStore::<LogId, Extensions>::new();
    DashManager::new(
        private_key.clone(),
        SpacesStore::new(),
        OpStore::new(op_store),
        rng,
    )
    .await
    .unwrap()
}
