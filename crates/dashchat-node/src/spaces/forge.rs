use p2panda_core::PublicKey;
use p2panda_spaces::traits::{AuthoredMessage, MessageStore};

use crate::{
    chat::BaseId,
    spaces::{SpaceControlMessage, SpacesArgs},
    stores::SpacesStore,
    timestamp_now,
};

#[derive(Clone, Debug)]
pub struct DashForge {
    pub public_key: PublicKey,
    pub store: SpacesStore,
}

impl p2panda_spaces::traits::Forge<BaseId, SpaceControlMessage, ()> for DashForge {
    type Error = anyhow::Error;

    fn public_key(&self) -> PublicKey {
        self.public_key
    }

    async fn forge(&self, args: SpacesArgs) -> Result<SpaceControlMessage, Self::Error> {
        let public_key = self.public_key;
        let message = SpaceControlMessage::new(public_key.into(), timestamp_now(), args)?;
        self.store.set_message(&message.id(), &message).await?;

        Ok(message)
    }
}
