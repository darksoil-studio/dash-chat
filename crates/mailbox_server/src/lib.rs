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

mod store_message;
mod get_messages;

pub use store_message::{StoreMessageRequest, store_message};
pub use get_messages::{GetMessagesRequest, GetMessagesResponse, get_messages_for_topics};

pub type TopicId = String;
pub type Message = Vec<u8>;

pub const MESSAGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("messages");

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
        let _table = write_txn.open_table(MESSAGES_TABLE)?;
    }
    write_txn.commit()?;

    tracing::info!("Database initialized successfully");

    Ok(db)
}

pub fn create_app(db: Database) -> Router {
    let state = AppState {
        db: Arc::new(db),
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/messages/store", post(store_message))
        .route("/messages/get", post(get_messages_for_topics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
