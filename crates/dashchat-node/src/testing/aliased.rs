use std::{collections::HashMap, sync::LazyLock, sync::Mutex};

use p2panda_spaces::traits::AuthoredMessage;

use crate::{ShortId, spaces::SpaceControlMessage};

static ALIASES: LazyLock<Mutex<HashMap<Vec<u8>, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn alias_space_messages<'a>(
    prefix: &str,
    msgs: impl IntoIterator<Item = &'a SpaceControlMessage>,
) {
    for (i, msg) in msgs.into_iter().enumerate() {
        msg.id()
            .aliased(format!("{prefix}/{i}/{:?}", msg.arg_type()).as_str());
    }
}

pub trait AliasedId: ShortId {
    const SHOW_SHORT_ID: bool = true;

    fn as_bytes(&self) -> &[u8];

    fn aliased(self, alias: &str) -> Self
    where
        Self: Sized,
    {
        let existing = ALIASES.lock().unwrap().insert(
            self.as_bytes().to_vec(),
            if Self::SHOW_SHORT_ID {
                format!("⟪{}|{}⟫", self.short(), alias)
            } else {
                format!("⟪{}‖{}⟫", <Self as ShortId>::PREFIX, alias)
            },
        );
        if let Some(existing) = existing {
            tracing::warn!(?existing, "alias already exists, replacing");
        }
        self
    }

    fn alias(&self) -> String {
        ALIASES
            .lock()
            .unwrap()
            .get(self.as_bytes())
            .cloned()
            .unwrap_or(self.short())
    }
}

impl AliasedId for p2panda_core::Hash {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AliasedId for p2panda_core::PublicKey {
    const SHOW_SHORT_ID: bool = false;

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AliasedId for p2panda_spaces::OperationId {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AliasedId for crate::chat::ChatId {
    const SHOW_SHORT_ID: bool = false;

    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}
