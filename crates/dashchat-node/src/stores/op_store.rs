use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::{Arc, RwLock},
};

use p2panda_core::{Body, Hash, Operation, PublicKey, RawOperation};
use p2panda_store::{LogStore, MemoryStore, OperationStore};
use p2panda_stream::operation::IngestResult;
use tokio::sync::Mutex;

use crate::{
    node::Orderer,
    payload::{Extensions, Payload},
    spaces::SpaceOperation,
    topic::{LogId, Topic, TopicKind},
    *,
};

#[derive(Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct OpStore {
    #[deref]
    #[deref_mut]
    pub(crate) store: MemoryStore<LogId, Extensions>,
    pub orderer: Arc<tokio::sync::RwLock<Orderer>>,
    pub processed_ops: Arc<RwLock<HashMap<LogId, HashSet<Hash>>>>,
    write_mutex: Arc<Mutex<()>>,
}

impl OpStore {
    pub fn new(store: MemoryStore<LogId, Extensions>) -> Self {
        let orderer = Arc::new(tokio::sync::RwLock::new(Orderer::new(
            store.clone(),
            Default::default(),
        )));

        Self {
            store,
            orderer,
            write_mutex: Arc::new(Mutex::new(())),
            processed_ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn author_operation<K: TopicKind>(
        &self,
        private_key: &PrivateKey,
        topic: Topic<K>,
        payload: Payload,
        deps: Vec<p2panda_core::Hash>,
        alias: Option<&str>,
    ) -> Result<(Header, Option<Body>), anyhow::Error> {
        let public_key = private_key.public_key();
        let log_id = topic.clone();

        let body = Some(payload.try_into_body()?);

        let extensions = Extensions {
            log_id: log_id.clone().into(),
        };

        let lock = self.write_mutex.lock().await;
        let latest_operation = self.latest_operation(&public_key, &log_id.into()).await?;

        let (seq_num, backlink) = match latest_operation {
            Some((header, _)) => (header.seq_num + 1, Some(header.hash())),
            None => (0, None),
        };

        let timestamp = timestamp_now();

        let mut header = Header {
            version: 1,
            public_key,
            signature: None,
            payload_size: body.as_ref().map_or(0, |body| body.size()),
            payload_hash: body.as_ref().map(|body| body.hash()),
            timestamp,
            seq_num,
            backlink,
            previous: deps,
            extensions,
        };

        header.sign(private_key);

        let log_id = header.extensions.log_id;
        let hash = header.hash();

        if let Some(alias) = alias {
            header.hash().with_name(alias);
        } else {
            header.hash().with_serial();
        }

        tracing::info!(
            log_id = ?log_id.renamed(),
            hash = ?hash.renamed(),
            seq_num = header.seq_num,
            "PUB: authoring operation"
        );

        let result = p2panda_stream::operation::ingest_operation(
            &mut *self.clone(),
            header.clone(),
            body.clone(),
            header.to_bytes(),
            &log_id.into(),
            false,
        )
        .await?;

        match result {
            IngestResult::Complete(op @ Operation { hash: hash2, .. }) => {
                assert_eq!(hash, hash2);

                // NOTE: if we fail to process here, incoming operations will be stuck as pending!
                self.process_ordering(op.clone()).await?;
            }

            IngestResult::Retry(h, _, _, missing) => {
                let backlink = h.backlink.as_ref().map(|h| h.renamed());
                tracing::error!(
                    ?log_id,
                    hash = ?hash.renamed(),
                    ?backlink,
                    ?missing,
                    "operation could not be ingested"
                );
                panic!("operation could not be ingested, check your sequence numbers!");
            }

            IngestResult::Outdated(op) => {
                tracing::error!(?op, "operation is outdated");
                panic!("operation is outdated");
            }
        }

        // Let the next op be authored as soon as this one's ingested
        drop(lock);

        Ok((header, body))
    }

    // SAM: could be generic https://github.com/p2panda/p2panda/blob/65727c7fff64376f9d2367686c2ed5132ff7c4e0/p2panda-stream/src/ordering/partial/mod.rs#L83
    pub async fn process_ordering(&self, operation: Operation<Extensions>) -> anyhow::Result<()> {
        self.orderer.write().await.process(operation).await?;
        Ok(())
    }

    pub async fn next_ordering(&self) -> anyhow::Result<Vec<Operation<Extensions>>> {
        let mut ordering = self.orderer.write().await;
        let mut next = vec![];
        while let Some(op) = ordering.next().await? {
            next.push(op);
        }
        Ok(next)
    }

    pub fn report<'a>(&self, log_ids: impl IntoIterator<Item = &'a LogId>) -> String {
        let log_ids = log_ids.into_iter().collect::<Vec<_>>();
        let s = self.store.read_store();
        let mut ops = s
            .operations
            .iter()
            .filter(|(_, (l, _, _, _))| {
                log_ids.is_empty() || log_ids.iter().find(|log_id| **log_id == l).is_some()
            })
            .collect::<Vec<_>>();
        ops.sort_by_key(|(_, (t, header, _, _))| (t, header.public_key.renamed(), header.seq_num));
        ops.into_iter()
            .map(|(h, (t, header, body, _))| {
                let desc = match body
                    .clone()
                    .map(|body| Payload::try_from_body(&body).unwrap())
                {
                    Some(Payload::Space(args)) => {
                        let space_op = SpaceOperation::new(header.clone(), args);
                        format!("{:?}", space_op.arg_type())
                    }
                    Some(p) => format!("{p:?}"),
                    None => "_".to_string(),
                };
                if log_ids.len() == 1 {
                    format!(
                        "• {} {:2} {} : {}",
                        header.public_key.renamed(),
                        header.seq_num,
                        h.renamed(),
                        desc
                    )
                } else {
                    let t = format!("{t:?}");
                    format!(
                        "• {:>24} {} {:2} {} : {}",
                        t,
                        header.public_key.renamed(),
                        header.seq_num,
                        h.renamed(),
                        desc
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn mark_op_processed(&self, log_id: LogId, hash: &Hash) {
        self.processed_ops
            .write()
            .unwrap()
            .entry(log_id)
            .or_default()
            .insert(hash.clone());
    }

    pub fn is_op_processed(&self, log_id: &LogId, hash: &Hash) -> bool {
        self.processed_ops
            .read()
            .unwrap()
            .get(log_id)
            .map(|s| s.contains(hash))
            .unwrap_or(false)
    }
}

impl OperationStore<LogId, Extensions> for OpStore {
    type Error = Infallible;

    async fn insert_operation(
        &mut self,
        hash: Hash,
        header: &Header,
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
    ) -> Result<Option<(Header, Option<Body>)>, Self::Error> {
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
    ) -> Result<Option<Vec<(Header, Option<Body>)>>, Self::Error> {
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
    ) -> Result<Option<(Header, Option<Body>)>, Self::Error> {
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

    async fn get_log_size(
        &self,
        public_key: &PublicKey,
        log_id: &LogId,
        from: Option<u64>,
    ) -> Result<Option<u64>, Self::Error> {
        self.store.get_log_size(public_key, log_id, from).await
    }

    async fn get_log_hashes(
        &self,
        public_key: &PublicKey,
        log_id: &LogId,
        from: Option<u64>,
    ) -> Result<Option<Vec<Hash>>, Self::Error> {
        self.store.get_log_hashes(public_key, log_id, from).await
    }
}
