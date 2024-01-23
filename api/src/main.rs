#![forbid(unsafe_code)]

use axum::http::header::InvalidHeaderValue;
use clap::Parser;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::try_join;
use tracing::error;

mod api;
mod config;
mod db;
mod devices;
mod program;

#[derive(Parser)]
struct Opt {
    /// Use a mock device instead of the real Arduino Brewslave
    #[clap(long)]
    use_mock: bool,
}

/// Possible API errors.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Address parse failed: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("Brew is ongoing")]
    BrewOngoing,
    #[error("Serial communication error: {0}")]
    CommError(#[from] comm::Error),
    #[error("Could not read configuration: {0}")]
    ConfigurationError(#[from] toml::de::Error),
    #[error("Internal error: {0}")]
    RecvError(#[from] oneshot::error::RecvError),
    #[error("Invalid header: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("JSON parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("Database problem: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}

/// API result type.
pub type Result<T, E = AppError> = std::result::Result<T, E>;

async fn try_main() -> Result<()> {
    let opts = Opt::parse();
    let config = config::Config::new()?;

    let (device_tx, device_rx) = mpsc::channel(32);
    let (brew_tx, brew_rx) = mpsc::channel(32);

    let db = db::Database::new(config.database).await?;
    let brew_future = program::run(device_tx.clone(), brew_rx, db.clone());
    let state = api::AppState::new(db, device_tx, brew_tx).await?;
    let server_future = api::run(state);

    if opts.use_mock {
        let device = devices::mock::Mock::new();
        let comm_future = devices::run(device, device_rx);
        try_join!(server_future, comm_future, brew_future)?;
    } else {
        let device = devices::brewslave::Brewslave::new(&config.device)?;
        let comm_future = devices::run(device, device_rx);
        try_join!(server_future, comm_future, brew_future)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(err) = try_main().await {
        error!("{}", err);
        std::process::exit(1);
    }
}
