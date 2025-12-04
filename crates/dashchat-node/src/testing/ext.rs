use named_id::*;
use p2panda_spaces::traits::AuthoredMessage;

use crate::{Topic, spaces::SpaceOperation, topic::TopicKind};

/// Add an alias to each space message with uniform naming.
/// Useful for debugging.
pub fn alias_space_messages<'a, K: TopicKind>(
    prefix: &str,
    _id: Topic<K>,
    msgs: impl IntoIterator<Item = &'a SpaceOperation>,
) {
    for (i, msg) in msgs.into_iter().enumerate() {
        let author = msg.author().renamed();
        let arg_type = msg.arg_type();
        msg.id()
            .with_name(format!("{prefix}/{author}/{i}/{arg_type:?}").as_str());
    }
}
