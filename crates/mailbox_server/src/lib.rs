use std::path::PathBuf;
use axum::{
    routing::{get, post},
    Router,
    Json,
};
use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};

mod blob;
mod store_blobs;
mod get_blobs;
mod cleanup;
mod watermark;

#[cfg(feature = "test_utils")]
pub mod test_utils;

pub use blob::Blob;
pub use store_blobs::{StoreBlobsRequest, store_blobs};
pub use get_blobs::{GetBlobsRequest, GetBlobsResponse, get_blobs_for_topics};
pub use cleanup::spawn_cleanup_task;
pub use watermark::compute_initial_watermarks;

pub type TopicId = String;
pub type LogId = String;
pub type SequenceNumber = u64;

// Database key format: "topic_id:log_id:sequence_number:uuid_v7"
// The UUID v7 suffix is used for cleanup based on message age
pub const BLOBS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("blobs");

// Watermarks table: tracks highest contiguous sequence number per topic:log
// Key format: "topic_id:log_id"
// Value: highest contiguous sequence number (0..=watermark are all present)
pub const WATERMARKS_TABLE: TableDefinition<&str, u64> = TableDefinition::new("watermarks");

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub fn init_db(db_path: PathBuf) -> Result<Database, Box<dyn std::error::Error>> {
    tracing::info!("Opening redb database at {:?}", db_path);

    let db = Database::create(&db_path)?;

    let write_txn = db.begin_write()?;
    {
        let _blobs_table = write_txn.open_table(BLOBS_TABLE)?;
        let _watermarks_table = write_txn.open_table(WATERMARKS_TABLE)?;
    }
    write_txn.commit()?;

    // Compute initial watermarks from existing blobs
    compute_initial_watermarks(&db)?;

    tracing::info!("Database initialized successfully");

    Ok(db)
}

pub fn create_app(db: Database) -> Router {
    create_app_with_arc(Arc::new(db))
}

pub fn create_app_with_arc(db: Arc<Database>) -> Router {
    let state = AppState { db };

    Router::new()
        .route("/health", get(health_check))
        .route("/blobs/store", post(store_blobs))
        .route("/blobs/get", post(get_blobs_for_topics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
