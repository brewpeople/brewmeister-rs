use axum::body::{Bytes, Full};
use axum::extract::Extension;
use axum::http::{Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{AddExtensionLayer, Json, Router};
use clap::Parser;
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::try_join;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Origin};
use tracing::{error, instrument};

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
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("JSON parse error")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    device: Arc<RwLock<models::Device>>,
    db: db::Database,
}

#[instrument]
async fn get_state(Extension(state): Extension<State>) -> Json<models::Device> {
    Json(state.device.read().await.clone())
}

#[instrument]
async fn get_recipes(Extension(state): Extension<State>) -> Json<models::Recipes> {
    Json(state.db.recipes().await)
}

#[instrument]
async fn post_recipe(Json(payload): Json<models::Recipe>, Extension(state): Extension<State>) {
    state
        .db
        .add_recipe(payload)
        .await
        .expect("do not fail now, handle me later");
}

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let tuple = match self {
            Self::IoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::ParseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        tuple.into_response()
    }
}

/// Start the web server.
#[instrument]
async fn run_server(state: State) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Origin::exact("http://0.0.0.0:8080".parse()?))
        .allow_methods(vec![Method::GET, Method::POST]);

    let app = Router::new()
        .route("/state", get(get_state))
        .route("/recipes", get(get_recipes).post(post_recipe))
        .layer(cors)
        .layer(ServiceBuilder::new().layer(AddExtensionLayer::new(state)));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[instrument]
async fn communicate<D>(device: D, state: State) -> anyhow::Result<()>
where
    D: crate::devices::Device + std::fmt::Debug,
{
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        interval.tick().await;

        let mut state = state.device.write().await;

        match device.read().await {
            Ok(new) => {
                state.current_temperature = new.current_temperature;
                state.target_temperature = new.current_temperature;
                state.stirrer_on = new.stirrer_on;
                state.heater_on = new.heater_on;
                state.serial_problem = false;
            }
            Err(err) => {
                error!("Error reading from device: {}", err);
                state.serial_problem = true;
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opt::parse();

    let state = State {
        device: Arc::new(RwLock::new(models::Device::default())),
        db: db::Database::new("db.json")?,
    };

    let server_future = run_server(state.clone());

    if opts.use_mock {
        let device = crate::devices::mock::Mock::new();
        let comm_future = communicate(device, state);
        try_join!(server_future, comm_future)?;
    } else {
        let device = crate::devices::brewslave::Brewslave::new()?;
        let comm_future = communicate(device, state);
        try_join!(server_future, comm_future)?;
    }

    Ok(())
}
