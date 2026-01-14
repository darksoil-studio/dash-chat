use std::collections::{BTreeMap, HashMap};

use mailbox_server::{Blob, GetBlobsRequest, GetBlobsResponse, StoreBlobsRequest};
use p2panda_core::{cbor::{decode_cbor, encode_cbor}, PublicKey};

use super::*;
use crate::{DeviceId, Topic, topic::LogId};

/// A client for the toy mailbox server.
#[derive(Clone)]
pub struct ToyMailboxClient {
    client: reqwest::Client,
    base_url: String,
}

impl ToyMailboxClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }
}

#[async_trait::async_trait]
impl MailboxClient for ToyMailboxClient {
    async fn publish(&self, ops: Vec<MailboxOperation>) -> Result<(), anyhow::Error> {
        if ops.is_empty() {
            return Ok(());
        }

        // Group operations by topic -> author -> seq_num
        let mut blobs: BTreeMap<String, BTreeMap<String, BTreeMap<u64, Blob>>> = BTreeMap::new();

        for op in ops {
            let topic_id = log_id_to_topic_id(&op.topic());
            let log_id = device_id_to_log_id(&op.author());
            let seq_num = op.seq_num();
            let blob = serialize_operation(&op)?;

            blobs
                .entry(topic_id)
                .or_default()
                .entry(log_id)
                .or_default()
                .insert(seq_num, blob);
        }

        let request = StoreBlobsRequest { blobs };
        let response = self
            .client
            .post(format!("{}/blobs/store", self.base_url))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to store blobs: {} - {}", status, body))
        }
    }

    async fn fetch(
        &self,
        request: FetchRequest<MailboxOperation>,
    ) -> Result<FetchResponse<MailboxOperation>, anyhow::Error> {
        // Convert FetchRequest to GetBlobsRequest
        let mut topics: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();

        for (log_id, authors) in request.0.iter() {
            let topic_id = log_id_to_topic_id(log_id);
            let mut log_map: BTreeMap<String, u64> = BTreeMap::new();

            for (device_id, height) in authors.iter() {
                let server_log_id = device_id_to_log_id(device_id);
                log_map.insert(server_log_id, *height);
            }

            topics.insert(topic_id, log_map);
        }

        let get_request = GetBlobsRequest { topics };
        let response = self
            .client
            .post(format!("{}/blobs/get", self.base_url))
            .json(&get_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to fetch blobs: {} - {}", status, body));
        }

        let response = response.json::<GetBlobsResponse>().await?;

        // Convert GetBlobsResponse to FetchResponse
        let mut result: BTreeMap<LogId, FetchTopicResponse<MailboxOperation>> = BTreeMap::new();

        for (topic_id_str, topic_response) in response.blobs_by_topic {
            let log_id = log_id_from_string(&topic_id_str)?;

            // Deserialize blobs to operations
            let mut ops = Vec::new();
            for (_author_str, seq_blobs) in topic_response.blobs {
                for (_seq, blob) in seq_blobs {
                    ops.push(deserialize_operation(&blob)?);
                }
            }

            // Convert missing map
            let mut missing: HashMap<DeviceId, Vec<u64>> = HashMap::new();
            for (author_str, seq_nums) in topic_response.missing {
                let device_id = device_id_from_string(&author_str)?;
                missing.insert(device_id, seq_nums);
            }

            result.insert(log_id, FetchTopicResponse { ops, missing });
        }

        Ok(FetchResponse(result))
    }
}

/// Helper functions

fn log_id_to_topic_id(log_id: &LogId) -> String {
    hex::encode(&**log_id)
}

fn device_id_to_log_id(device_id: &DeviceId) -> String {
    hex::encode(device_id.as_bytes())
}

fn log_id_from_string(s: &str) -> Result<LogId, anyhow::Error> {
    let topic: Topic = s.parse()?;
    Ok(topic.into())
}

fn device_id_from_string(s: &str) -> Result<DeviceId, anyhow::Error> {
    let bytes = hex::decode(s)?;
    let pk = PublicKey::try_from(bytes.as_slice())?;
    Ok(pk.into())
}

fn serialize_operation(op: &MailboxOperation) -> Result<Blob, anyhow::Error> {
    let bytes = encode_cbor(op)?;
    Ok(Blob::new(bytes))
}

fn deserialize_operation(blob: &Blob) -> Result<MailboxOperation, anyhow::Error> {
    Ok(decode_cbor(blob.as_slice())?)
}
