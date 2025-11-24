use p2panda_core::{Operation, cbor::EncodeError};
use p2panda_spaces::OperationId;
use serde::{Deserialize, Serialize};

pub type SpacesArgs = p2panda_spaces::SpacesArgs<ChatId, ()>;

use crate::{AsBody, Extensions, Header, Payload};

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpaceOperation {
    pub header: Header,
    pub args: SpacesArgs,
}

impl p2panda_spaces::traits::AuthoredMessage for SpaceOperation {
    fn id(&self) -> OperationId {
        OperationId::from(self.header.hash())
    }

    fn author(&self) -> p2panda_spaces::ActorId {
        self.header.public_key.into()
    }
}

impl p2panda_spaces::traits::SpacesMessage<ChatId, ()> for SpaceOperation {
    fn args(&self) -> &SpacesArgs {
        &self.args
    }
}

// SAM: Operation goes inside here, not the other way around!
// SAM: SpacesArgs should probably go on the Extensions
impl SpaceOperation {
    pub fn new(header: Header, args: SpacesArgs) -> Self {
        Self { header, args }
    }

    pub fn from_payload(header: &Header, payload: &Payload) -> anyhow::Result<Option<Self>> {
        match payload {
            Payload::Space(args) => Ok(Some(Self::new(header.clone(), args.clone()))),
            _ => Ok(None),
        }
    }

    pub fn into_operation(self) -> anyhow::Result<Operation<Extensions>> {
        Ok(Operation {
            header: self.header.clone(),
            hash: self.header.hash(),
            body: Some(Payload::Space(self.args.clone()).try_into_body()?),
        })
    }

    pub fn timestamp(&self) -> u64 {
        self.header.timestamp
    }

    pub fn dependencies(&self) -> Vec<OperationId> {
        match &self.args {
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
        match &self.args {
            p2panda_spaces::SpacesArgs::KeyBundle { .. } => ArgType::KeyBundle,
            p2panda_spaces::SpacesArgs::Auth { .. } => ArgType::Auth,
            p2panda_spaces::SpacesArgs::SpaceMembership { .. } => ArgType::SpaceMembership,
            p2panda_spaces::SpacesArgs::SpaceUpdate { .. } => ArgType::SpaceUpdate,
            p2panda_spaces::SpacesArgs::Application { .. } => ArgType::Application,
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
