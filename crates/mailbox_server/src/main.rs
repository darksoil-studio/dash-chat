use clap::Parser;
use mailbox_server::{create_app, init_db};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "mailbox_server")]
#[command(about = "A simple mailbox server for storing and retrieving messages", long_about = None)]
struct Args {
    /// Path to the redb database file
    #[arg(short, long, default_value = "mailbox.redb")]
    db_path: String,

    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:3000")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mailbox_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let db = init_db(args.db_path.into())?;
    let app = create_app(db);

    let listener = tokio::net::TcpListener::bind(&args.addr).await?;
    let addr = listener.local_addr()?;

    tracing::info!("Mailbox server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
