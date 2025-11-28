use std::{collections::HashMap, sync::LazyLock, sync::Mutex};

use p2panda_spaces::traits::AuthoredMessage;

use crate::{ChatId, Topic, spaces::SpaceOperation, testing::ShortId, topic::TopicKind};

static ALIASES: LazyLock<Mutex<HashMap<Vec<u8>, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Add an alias to each space message with uniform naming.
/// Useful for debugging.
pub fn alias_space_messages<'a, K: TopicKind>(
    prefix: &str,
    id: Topic<K>,
    msgs: impl IntoIterator<Item = &'a SpaceOperation>,
) {
    for (i, msg) in msgs.into_iter().enumerate() {
        let author = msg.author().alias();
        let arg_type = msg.arg_type();
        msg.id()
            .aliased(format!("{prefix}/{author}/{id:?}/{i}/{arg_type:?}").as_str());
    }
}

pub trait AliasedId: ShortId {
    const SHOW_SHORT_ID: bool = true;

    fn as_bytes(&self) -> &[u8];

    fn aliased(self, alias: &str) -> Self
    where
        Self: Sized,
    {
        let alias = if Self::SHOW_SHORT_ID {
            format!("⟪{}|{}⟫", self.short(), alias)
        } else {
            format!("⟪{}‖{}⟫", <Self as ShortId>::prefix(), alias)
        };
        let existing = ALIASES
            .lock()
            .unwrap()
            .insert(self.as_bytes().to_vec(), alias.clone());
        if let Some(existing) = existing {
            if existing != alias {
                tracing::warn!(?existing, ?alias, "alias already exists, replacing");
            }
        }
        self
    }

    fn alias(&self) -> String {
        ALIASES
            .lock()
            .unwrap()
            .get(self.as_bytes())
            .cloned()
            .unwrap_or_else(|| self.default_alias())
    }

    fn default_alias(&self) -> String {
        format!("⟪{}⟫", self.short())
    }
}
