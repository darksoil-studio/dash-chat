use p2panda_core::{PrivateKey, PublicKey};
use p2panda_spaces::traits::{AuthoredMessage, MessageStore};

use crate::{
    Payload, Topic,
    chat::ChatId,
    spaces::{SpaceOperation, SpacesArgs},
    stores::{OpStore, SpacesStore},
    topic::{LogId, kind},
};

#[derive(Clone, derive_more::Debug)]
#[debug("DashForge")]
pub struct DashForge {
    pub private_key: PrivateKey,
    pub store: SpacesStore,
    pub op_store: OpStore,
}

impl p2panda_spaces::traits::Forge<ChatId, SpaceOperation, ()> for DashForge {
    type Error = anyhow::Error;

    async fn forge(&self, args: SpacesArgs) -> Result<SpaceOperation, Self::Error> {
        let topic: Topic<kind::Untyped> = match &args {
            p2panda_spaces::SpacesArgs::KeyBundle { key_bundle } => unimplemented!(),
            p2panda_spaces::SpacesArgs::Auth {
                control_message, ..
            } => Topic::announcements(control_message.group_id).recast(),
            p2panda_spaces::SpacesArgs::SpaceMembership {
                space_id, group_id, ..
            } => space_id.recast(),
            p2panda_spaces::SpacesArgs::SpaceUpdate {
                space_id, group_id, ..
            } => space_id.recast(),
            p2panda_spaces::SpacesArgs::Application { space_id, .. } => space_id.recast(),
        };
        let (header, _) = crate::node::author_operation::create_operation(
            &self.op_store,
            &self.private_key,
            topic,
            Payload::Space(args.clone()),
            vec![],
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
