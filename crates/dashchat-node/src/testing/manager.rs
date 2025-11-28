use crate::{
    Extensions,
    spaces::DashManager,
    stores::{OpStore, SpacesStore},
    topic::LogId,
};

use p2panda_core::PrivateKey;
use p2panda_encryption::Rng;
use p2panda_spaces::ActorId;

// pub async fn test_manager() -> DashManager {
//     let rng = Rng::default();
//     let private_key = PrivateKey::new();

//     let chat_actor_id: ActorId = PrivateKey::from_bytes(&rng.random_array()?)
//         .public_key()
//         .into();

//     let op_store = p2panda_store::MemoryStore::<LogId, Extensions>::new();
//     DashManager::new(
//         private_key.clone(),
//         chat_actor_id.clone(),
//         SpacesStore::new(),
//         OpStore::new(op_store),
//         rng,
//     )
//     .await
//     .unwrap()
// }
