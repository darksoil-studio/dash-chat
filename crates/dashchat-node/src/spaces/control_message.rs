use p2panda_core::cbor::{EncodeError, encode_cbor};
use p2panda_spaces::OperationId;
use serde::{Deserialize, Serialize};

pub type SpacesArgs = p2panda_spaces::message::SpacesArgs<ChatId, ()>;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpaceControlMessage {
    pub hash: p2panda_core::Hash,
    pub author: p2panda_spaces::ActorId,
    pub timestamp: u64,
    pub spaces_args: SpacesArgs,
}

impl p2panda_spaces::message::AuthoredMessage for SpaceControlMessage {
    fn id(&self) -> OperationId {
        OperationId::from(self.hash.clone())
    }

    fn author(&self) -> p2panda_spaces::ActorId {
        self.author
    }
}

impl p2panda_spaces::message::SpacesMessage<ChatId, ()> for SpaceControlMessage {
    fn args(&self) -> &SpacesArgs {
        &self.spaces_args
    }
}

impl SpaceControlMessage {
    pub fn new(
        author: p2panda_spaces::ActorId,
        timestamp: u64,
        spaces_args: SpacesArgs,
    ) -> Result<Self, EncodeError> {
        let bytes = encode_cbor(&(author, &spaces_args)).unwrap();
        Ok(Self {
            hash: p2panda_core::Hash::new(bytes),
            author,
            timestamp,
            spaces_args,
        })
    }

    pub fn dependencies(&self) -> Vec<OperationId> {
        match &self.spaces_args {
            SpacesArgs::KeyBundle { .. } => vec![],
            SpacesArgs::SpaceMembership {
                space_dependencies,
                auth_message_id,
                ..
            } => [auth_message_id.clone()]
                .into_iter()
                .chain(space_dependencies.clone())
                .collect(),
            SpacesArgs::Auth {
                auth_dependencies, ..
            } => auth_dependencies.into_iter().cloned().collect::<Vec<_>>(),
            SpacesArgs::SpaceUpdate {
                space_dependencies, ..
            } => space_dependencies.into_iter().cloned().collect::<Vec<_>>(),
            SpacesArgs::Application {
                space_dependencies, ..
            } => space_dependencies.into_iter().cloned().collect::<Vec<_>>(),
        }
    }

    pub fn arg_type(&self) -> ArgType {
        match &self.spaces_args {
            p2panda_spaces::message::SpacesArgs::KeyBundle {} => ArgType::KeyBundle,
            p2panda_spaces::message::SpacesArgs::Auth {
                control_message,
                auth_dependencies,
            } => ArgType::Auth,
            p2panda_spaces::message::SpacesArgs::SpaceMembership {
                space_id,
                group_id,
                space_dependencies,
                auth_message_id,
                direct_messages,
            } => ArgType::SpaceMembership,
            p2panda_spaces::message::SpacesArgs::SpaceUpdate {
                space_id,
                group_id,
                space_dependencies,
            } => ArgType::SpaceUpdate,
            p2panda_spaces::message::SpacesArgs::Application {
                space_id,
                space_dependencies,
                group_secret_id,
                nonce,
                ciphertext,
            } => ArgType::Application,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgType {
    KeyBundle,
    Auth,
    SpaceMembership,
    SpaceUpdate,
    Application,
}
