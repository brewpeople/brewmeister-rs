use axum::body::{Bytes, Full};
use axum::extract::Extension;
use axum::http::{Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{AddExtensionLayer, Json, Router};
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::try_join;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Origin};
use tracing::{debug, instrument};

#[derive(Debug, Error)]
enum AppError {
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

#[derive(Clone, Debug, Default, Serialize)]
struct Inner {
    current_temperature: f32,
    target_temperature: f32,
    stirrer_on: bool,
    heater_on: bool,
}

impl From<comm::State> for Inner {
    fn from(s: comm::State) -> Self {
        Self {
            current_temperature: s.current_temperature,
            target_temperature: s.target_temperature,
            stirrer_on: s.stirrer_on,
            heater_on: s.heater_on,
        }
    }
}

#[derive(Clone, Debug)]
struct State {
    inner: Arc<RwLock<Inner>>,
}

#[instrument]
async fn get_state(Extension(state): Extension<State>) -> Json<Inner> {
    let state = state.inner.read().await;
    Json(state.clone())
}

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let tuple = match self {
            Self::IoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        tuple.into_response()
    }
}

/// Set up the serial connection and poll for new temperature, stirrer and heater values.
#[instrument]
async fn communicate(state: State) -> anyhow::Result<()> {
    let client = comm::Comm::new()?;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        interval.tick().await;

        let mut state = state.inner.write().await;
        let current = client.read_state().await?;
        *state = current.into();
        debug!("read {:?}", state);
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
        .layer(cors)
        .layer(ServiceBuilder::new().layer(AddExtensionLayer::new(state)));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let state = State {
        inner: Arc::new(RwLock::new(Inner::default())),
    };

    let server_future = run_server(state.clone());
    let comm_future = communicate(state);
    try_join!(server_future, comm_future)?;

    Ok(())
}
