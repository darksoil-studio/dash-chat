use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::{Arc, RwLock},
};

use p2panda_core::{Body, Hash, Header, PublicKey, RawOperation};
use p2panda_store::{LogStore, MemoryStore, OperationStore};

use crate::{
    network::{LogId, Topic},
    operation::Extensions,
    testing::AliasedId,
    *,
};

#[derive(Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct OpStore {
    #[deref]
    #[deref_mut]
    store: MemoryStore<LogId, Extensions>,
    pub processed_ops: Arc<RwLock<HashMap<Topic, HashSet<Hash>>>>,
}

impl OpStore {
    pub fn new(store: MemoryStore<LogId, Extensions>) -> Self {
        Self {
            store,
            processed_ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn report<'a>(&self, topics: impl IntoIterator<Item = &'a Topic>) -> String {
        let topics = topics.into_iter().collect::<HashSet<_>>();
        let s = self.store.read_store();
        let mut ops = s
            .operations
            .iter()
            .filter(|(_, (t, _, _, _))| topics.is_empty() || topics.contains(t))
            .collect::<Vec<_>>();
        ops.sort_by_key(|(_, (t, header, _, _))| (t, header.public_key.alias(), header.seq_num));
        ops.into_iter()
            .map(|(h, (t, header, body, _))| {
                let desc = match body
                    .clone()
                    .map(|body| Payload::try_from_body(body).unwrap())
                {
                    Some(Payload::SpaceControl(msgs)) => {
                        format!(
                            "{:?}",
                            msgs.iter().map(|m| m.arg_type()).collect::<Vec<_>>()
                        )
                    }
                    Some(Payload::Invitation(invitation)) => format!("{:?}", invitation),
                    None => "_".to_string(),
                };
                if topics.len() == 1 {
                    format!(
                        "• {} {:2} {} : {}",
                        header.public_key.alias(),
                        header.seq_num,
                        h.alias(),
                        desc
                    )
                } else {
                    let t = format!("{t:?}");
                    format!(
                        "• {:>24} {} {:2} {} : {}",
                        t,
                        header.public_key.alias(),
                        header.seq_num,
                        h.alias(),
                        desc
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn mark_op_processed(&self, topic: &Topic, hash: &Hash) {
        self.processed_ops
            .write()
            .unwrap()
            .entry(topic.clone())
            .or_default()
            .insert(hash.clone());
    }

    pub fn is_op_processed(&self, topic: &Topic, hash: &Hash) -> bool {
        self.processed_ops
            .read()
            .unwrap()
            .get(topic)
            .map(|s| s.contains(hash))
            .unwrap_or(false)
    }
}

impl OperationStore<LogId, Extensions> for OpStore {
    type Error = Infallible;

    async fn insert_operation(
        &mut self,
        hash: Hash,
        header: &Header<Extensions>,
        body: Option<&Body>,
        header_bytes: &[u8],
        log_id: &LogId,
    ) -> Result<bool, Self::Error> {
        self.store
            .insert_operation(hash, header, body, header_bytes, log_id)
            .await
    }

    async fn get_operation(
        &self,
        hash: Hash,
    ) -> Result<Option<(Header<Extensions>, Option<Body>)>, Self::Error> {
        self.store.get_operation(hash).await
    }

    async fn get_raw_operation(&self, hash: Hash) -> Result<Option<RawOperation>, Self::Error> {
        self.store.get_raw_operation(hash).await
    }

    async fn has_operation(&self, hash: Hash) -> Result<bool, Self::Error> {
        self.store.has_operation(hash).await
    }

    async fn delete_operation(&mut self, hash: Hash) -> Result<bool, Self::Error> {
        self.store.delete_operation(hash).await
    }

    async fn delete_payload(&mut self, hash: Hash) -> Result<bool, Self::Error> {
        self.store.delete_payload(hash).await
    }
}

impl LogStore<LogId, Extensions> for OpStore {
    type Error = Infallible;

    async fn get_log(
        &self,
        public_key: &PublicKey,
        log_id: &LogId,
        from: Option<u64>,
    ) -> Result<Option<Vec<(Header<Extensions>, Option<Body>)>>, Self::Error> {
        self.store.get_log(public_key, log_id, from).await
    }

    async fn get_raw_log(
        &self,
        public_key: &PublicKey,
        log_id: &LogId,
        from: Option<u64>,
    ) -> Result<Option<Vec<RawOperation>>, Self::Error> {
        self.store.get_raw_log(public_key, log_id, from).await
    }

    async fn latest_operation(
        &self,
        public_key: &PublicKey,
        log_id: &LogId,
    ) -> Result<Option<(Header<Extensions>, Option<Body>)>, Self::Error> {
        self.store.latest_operation(public_key, log_id).await
    }

    async fn get_log_heights(&self, log_id: &LogId) -> Result<Vec<(PublicKey, u64)>, Self::Error> {
        self.store.get_log_heights(log_id).await
    }

    async fn delete_operations(
        &mut self,
        public_key: &PublicKey,
        log_id: &LogId,
        before: u64,
    ) -> Result<bool, Self::Error> {
        self.store
            .delete_operations(public_key, log_id, before)
            .await
    }

    async fn delete_payloads(
        &mut self,
        public_key: &PublicKey,
        log_id: &LogId,
        from: u64,
        to: u64,
    ) -> Result<bool, Self::Error> {
        self.store
            .delete_payloads(public_key, log_id, from, to)
            .await
    }
}
