use crate::devices::Device;
use axum::http::header::InvalidHeaderValue;
use clap::Parser;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::try_join;
use tracing::error;

mod api;
mod db;
mod devices;

#[derive(Parser)]
struct Opt {
    /// Use a mock device instead of the real Arduino Brewslave
    #[clap(long)]
    use_mock: bool,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Address parse failed: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("Serial communication error: {0}")]
    CommError(#[from] comm::Error),
    #[error("Reading .env failed: {0}")]
    DotenvError(#[from] dotenv::Error),
    #[error("Hyper error: {0}")]
    HyperError(#[from] hyper::Error),
    #[error("Invalid header: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("JSON parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("Database problem: {0}")]
    SqlError(#[from] sqlx::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    device: Arc<RwLock<models::Device>>,
    db: db::Database,
}

async fn try_main() -> Result<(), AppError> {
    dotenv::dotenv()?;

    let opts = Opt::parse();

    let state = State {
        device: Arc::new(RwLock::new(models::Device::default())),
        db: db::Database::new().await?,
    };

    let server_future = api::run(state.clone());

    if opts.use_mock {
        let device = crate::devices::mock::Mock::new();
        let comm_future = device.run(state);
        try_join!(server_future, comm_future)?;
    } else {
        let device = crate::devices::brewslave::Brewslave::new()?;
        let comm_future = device.run(state);
        try_join!(server_future, comm_future)?;
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
