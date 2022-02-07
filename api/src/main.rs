use axum::body;
use axum::extract::{Extension, Path};
use axum::headers::{HeaderMap, HeaderValue};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{AddExtensionLayer, Json, Router};
use clap::Parser;
use include_dir::{include_dir, Dir};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::try_join;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Origin};
use tracing::{error, instrument, warn};

mod db;
mod devices;

static DIST_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../app/dist");

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
    #[error("Database problem: {0}")]
    SqlError(#[from] sqlx::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    device: Arc<RwLock<models::Device>>,
    db: db::Database,
}

fn insert_header_from_extension(map: &mut HeaderMap, ext: &str) {
    match ext {
        "css" => {
            map.insert(CONTENT_TYPE, HeaderValue::from_static("text/css"));
        }
        "html" => {
            map.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
        }
        "js" => {
            map.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/javascript"),
            );
        }
        "wasm" => {
            map.insert(CONTENT_TYPE, HeaderValue::from_static("application/wasm"));
        }
        _ => {}
    }
}

#[instrument]
async fn get_state(Extension(state): Extension<State>) -> Json<models::Device> {
    Json(state.device.read().await.clone())
}

#[instrument]
async fn get_recipes(
    Extension(state): Extension<State>,
) -> Result<Json<models::Recipes>, AppError> {
    Ok(Json(state.db.recipes().await?))
}

#[instrument]
async fn post_recipe(Json(payload): Json<models::Recipe>, Extension(state): Extension<State>) {
    state
        .db
        .add_recipe(payload)
        .await
        .expect("do not fail now, handle me later");
}

#[instrument]
async fn get_static(Path(path): Path<String>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();

    match DIST_DIR.get_file(&path) {
        Some(file) => {
            if let Some(e) = file.path().extension().map(|e| e.to_str()).flatten() {
                insert_header_from_extension(&mut headers, e);
            }

            (StatusCode::OK, headers, file.contents().to_vec())
        }
        None => {
            warn!("file not found");
            (StatusCode::NOT_FOUND, headers, vec![])
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::boxed(body::Full::from(format!("Error: {}", self))))
            .unwrap()
    }
}

/// Start the web server.
#[instrument]
async fn run_server(state: State) -> anyhow::Result<()> {
    // Only useful if we run the app via `trunk serve`, if not we serve the static files directly.
    let cors = CorsLayer::new()
        .allow_origin(Origin::exact("http://0.0.0.0:8080".parse()?))
        .allow_methods(vec![Method::GET, Method::POST]);

    let app = Router::new()
        .route(
            "/",
            get(|| async { get_static(Path("index.html".into())).await }),
        )
        .route("/:key", get(get_static))
        .route("/api/state", get(get_state))
        .route("/api/recipes", get(get_recipes).post(post_recipe))
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
        db: db::Database::new().await?,
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
