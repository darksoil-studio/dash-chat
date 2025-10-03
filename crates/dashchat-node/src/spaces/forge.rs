use p2panda_core::{PrivateKey, PublicKey};

use crate::{
    chat::ChatId,
    spaces::{SpaceControlMessage, SpacesArgs},
    timestamp_now,
};

#[derive(Clone, Debug)]
pub struct DashForge {
    pub private_key: PrivateKey,
}

impl p2panda_spaces::forge::Forge<ChatId, SpaceControlMessage, ()> for DashForge {
    type Error = anyhow::Error;

    fn public_key(&self) -> PublicKey {
        self.private_key.public_key()
    }

    async fn forge(&mut self, args: SpacesArgs) -> Result<SpaceControlMessage, Self::Error> {
        let public_key = self.private_key.public_key();
        Ok(SpaceControlMessage::new(
            public_key.into(),
            timestamp_now(),
            args,
        )?)
    }

    async fn forge_ephemeral(
        &mut self,
        private_key: PrivateKey,
        args: SpacesArgs,
    ) -> Result<SpaceControlMessage, Self::Error> {
        Ok(SpaceControlMessage {
            // TODO: is this ok?
            hash: p2panda_core::Hash::new([0; 32]),
            author: private_key.public_key().into(),
            timestamp: timestamp_now(),
            spaces_args: args,
        })
    }
}
