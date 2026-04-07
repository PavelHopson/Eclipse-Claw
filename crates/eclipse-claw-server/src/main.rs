mod error;
mod handlers;
mod routes;
mod state;

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

/// eclipse-claw REST API server
#[derive(Parser)]
#[command(name = "eclipse-claw-server", version, about)]
struct Args {
    /// Address to bind on
    #[arg(long, env = "ECLIPSE_SERVER_ADDR", default_value = "0.0.0.0:3000")]
    addr: String,

    /// Number of concurrent fetch connections allowed
    #[arg(long, env = "ECLIPSE_MAX_CONCURRENCY", default_value_t = 32)]
    max_concurrency: usize,

    /// Request body size limit in bytes (default 4 MB)
    #[arg(long, env = "ECLIPSE_BODY_LIMIT", default_value_t = 4 * 1024 * 1024)]
    body_limit: usize,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("eclipse_claw_server=debug,info")),
        )
        .init();

    let args = Args::parse();
    let state = state::AppState::new(args.max_concurrency).await;
    let app = routes::build(state, args.body_limit);

    tracing::info!(addr = %args.addr, "starting eclipse-claw-server");

    let listener = tokio::net::TcpListener::bind(&args.addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {}: {e}", args.addr));

    axum::serve(listener, app)
        .await
        .expect("server error");
}
