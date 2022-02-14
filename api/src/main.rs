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
    #[error("Serial communication error: {0}")]
    CommError(#[from] comm::Error),
    #[error("Could not read configuration: {0}")]
    ConfigurationError(#[from] toml::de::Error),
    #[error("Reading .env failed: {0}")]
    DotenvError(#[from] dotenv::Error),
    #[error("Hyper error: {0}")]
    HyperError(#[from] hyper::Error),
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
}

/// API result type.
pub type Result<T> = std::result::Result<T, AppError>;

async fn try_main() -> Result<()> {
    dotenv::dotenv()?;

    let opts = Opt::parse();
    let config = config::Config::new()?;

    let (device_tx, device_rx) = mpsc::channel(32);
    let (brew_tx, brew_rx) = mpsc::channel(32);

    let brew_future = program::run(device_tx.clone(), brew_rx);
    let state = api::State::new(device_tx, brew_tx).await?;
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
