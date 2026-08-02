mod error;
mod handlers;
mod routes;
mod state;
mod worker;

use std::net::SocketAddr;

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

/// eclipse-claw REST API server
#[derive(Parser)]
#[command(name = "eclipse-claw-server", version, about)]
struct Args {
    /// Address to bind on
    #[arg(long, env = "ECLIPSE_SERVER_ADDR", default_value = "127.0.0.1:3000")]
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
    let addr: SocketAddr = args
        .addr
        .parse()
        .unwrap_or_else(|e| panic!("invalid server address {}: {e}", args.addr));
    let server_token = std::env::var("ECLIPSE_SERVER_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty());

    validate_bind_security(addr, server_token.as_deref())
        .unwrap_or_else(|message| panic!("{message}"));
    let authentication = if server_token.is_some() {
        "bearer"
    } else {
        "loopback-only"
    };

    let state = state::AppState::new(args.max_concurrency)
        .await
        .unwrap_or_else(|message| panic!("server initialization failed: {message}"));
    let app = routes::build(state, args.body_limit, args.max_concurrency, server_token);

    tracing::info!(
        addr = %addr,
        authentication,
        "starting eclipse-claw-server"
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {addr}: {e}"));

    axum::serve(listener, app).await.expect("server error");
}

fn validate_bind_security(addr: SocketAddr, token: Option<&str>) -> Result<(), &'static str> {
    if addr.ip().is_loopback() {
        return Ok(());
    }

    if token.is_some_and(|value| value.len() >= 32 && !value.chars().any(char::is_whitespace)) {
        Ok(())
    } else {
        Err(
            "refusing unauthenticated external bind: set ECLIPSE_SERVER_TOKEN to at least 32 non-whitespace characters",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_bind_does_not_require_token() {
        assert!(validate_bind_security("127.0.0.1:3000".parse().unwrap(), None).is_ok());
    }

    #[test]
    fn external_bind_requires_strong_bearer_token() {
        let addr = "0.0.0.0:3000".parse().unwrap();
        assert!(validate_bind_security(addr, None).is_err());
        assert!(validate_bind_security(addr, Some("short")).is_err());
        assert!(
            validate_bind_security(addr, Some("contains whitespace and is still invalid x"))
                .is_err()
        );
        assert!(validate_bind_security(addr, Some("0123456789abcdef0123456789abcdef")).is_ok());
    }
}
