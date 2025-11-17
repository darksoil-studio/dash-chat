use crate::{spaces::DashManager, stores::SpacesStore};

use p2panda_core::PrivateKey;
use p2panda_encryption::Rng;

pub async fn test_manager() -> DashManager {
    let rng = Rng::default();
    let private_key = PrivateKey::new();
    DashManager::new(private_key.clone(), SpacesStore::new(), rng)
        .await
        .unwrap()
}
