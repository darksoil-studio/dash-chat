use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use p2panda_core::PublicKey;
use tokio::sync::RwLock;

use async_trait::async_trait;
use p2panda_sync::log_sync::TopicLogMap;

use crate::PK;
use crate::chat::{DirectId, GroupId, InboxId};
use p2panda_net::TopicId;
use p2panda_sync::TopicQuery;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    derive_more::Display,
)]
#[display("{:?}", self)]
pub enum Topic {
    /// Announcements include changes to the node's profile (name+avatar).
    ///
    /// This is backed by a Space where the Author is the only Manager,
    /// and anyone else added has only Read access.
    Announcements(PK),

    /// The inbox is used for things like friend requests and group invitations
    /// from people not yet my friend.
    ///
    /// This is backed by a Space where the Author is the only Manager,
    /// and anyone else added has only Read access.
    Inbox(InboxId),

    /// A symmetric topic for communication between exactly two keys.
    /// Used for direct messages as well as friend requests and group invitations.
    Direct(DirectId),

    /// A group chat ()
    Group(GroupId),
}

impl TopicQuery for Topic {}

impl TopicId for Topic {
    fn id(&self) -> [u8; 32] {
        match self {
            Topic::Announcements(public_key) => *public_key.as_bytes(),
            Topic::Inbox(chat_id) => ***chat_id,
            Topic::Direct(chat_id) => ***chat_id,
            Topic::Group(chat_id) => ***chat_id,
        }
    }
}

pub type LogId = Topic;

#[derive(Clone, Debug)]
pub struct AuthorStore<T>(pub(crate) Arc<RwLock<HashMap<T, HashSet<PK>>>>);

impl<T: Eq + std::hash::Hash + std::fmt::Debug + Clone> AuthorStore<T> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub async fn add_author(&self, topic: T, public_key: impl Into<PK>) {
        let mut authors = self.0.write().await;
        let public_key = public_key.into();
        let pk = PK::from(public_key);

        authors
            .entry(topic.clone())
            .and_modify(|public_keys| {
                if public_keys.insert(public_key) {
                    tracing::debug!(?topic, ?pk, "added author");
                }
            })
            .or_insert({
                tracing::debug!(?topic, ?pk, "added author (first in topic)");
                let mut public_keys = HashSet::new();
                public_keys.insert(public_key);
                public_keys
            });
    }

    pub async fn authors(&self, topic: &T) -> Option<HashSet<PK>> {
        let authors = self.0.read().await;
        Some(
            authors
                .get(topic)
                .cloned()?
                .into_iter()
                .map(PK::from)
                .collect(),
        )
    }
}

#[async_trait]
impl<Topic: Eq + std::hash::Hash + TopicQuery> TopicLogMap<Topic, Topic> for AuthorStore<Topic> {
    /// During sync other peers are interested in all our append-only logs for a certain topic.
    /// This method tells the sync protocol which logs we have available from which author for that
    /// given topic.
    async fn get(&self, topic: &Topic) -> Option<HashMap<PublicKey, Vec<Topic>>> {
        let authors = self.authors(topic).await;
        let map = match authors {
            Some(authors) => {
                let mut map = HashMap::with_capacity(authors.len());
                for author in authors {
                    // We write all data of one author into one log for now.
                    map.insert(author.into(), vec![topic.clone()]);
                }
                map
            }
            None => HashMap::new(),
        };
        Some(map)
    }
}
