use mailbox_server::{create_app, init_db};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mailbox_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = init_db()?;
    let app = create_app(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let addr = listener.local_addr()?;

    tracing::info!("Mailbox server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
