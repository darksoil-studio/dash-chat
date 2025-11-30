use std::sync::Arc;

use p2panda_core::{PrivateKey, PublicKey};
use p2panda_spaces::{
    ActorId,
    traits::{AuthoredMessage, MessageStore},
};

use crate::{
    Payload, Topic,
    chat::ChatId,
    spaces::{SpaceOperation, SpacesArgs},
    stores::{OpStore, SpacesStore},
    topic::kind,
};

#[derive(Clone, derive_more::Debug)]
#[debug("DashForge")]
pub struct DashForge {
    pub private_key: PrivateKey,
    pub chat_actor_id: ActorId,
    pub store: SpacesStore,
    pub op_store: OpStore,
}

impl p2panda_spaces::traits::Forge<ChatId, SpaceOperation, ()> for DashForge {
    type Error = anyhow::Error;

    async fn forge(&self, args: SpacesArgs) -> Result<SpaceOperation, Self::Error> {
        // TODO: this is crucial, need to set the correct topic for each op.
        let topic: Topic<kind::Untyped> = match &args {
            p2panda_spaces::SpacesArgs::KeyBundle { key_bundle } => Topic::global().recast(),
            p2panda_spaces::SpacesArgs::Auth {
                control_message, ..
            } => Topic::global().recast(),
            p2panda_spaces::SpacesArgs::SpaceMembership {
                space_id, group_id, ..
            } => Topic::global().recast(),
            p2panda_spaces::SpacesArgs::SpaceUpdate {
                space_id, group_id, ..
            } => Topic::global().recast(),
            p2panda_spaces::SpacesArgs::Application { space_id, .. } => Topic::global().recast(),
        };
        let (header, _) = self
            .op_store
            .author_operation(
                &self.private_key,
                topic,
                Payload::Space(args.clone()),
                vec![],
                None,
            )
            .await?;
        let message = SpaceOperation::new(header, args);
        self.store.set_message(&message.id(), &message).await?;

        Ok(message)
    }

    fn public_key(&self) -> PublicKey {
        self.private_key.public_key()
    }
}
