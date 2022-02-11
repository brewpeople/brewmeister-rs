use crate::{db, devices, AppError};
use axum::body;
use axum::extract::{Extension, Path};
use axum::headers::{HeaderMap, HeaderValue};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{AddExtensionLayer, Json, Router};
use include_dir::{include_dir, Dir};
use tokio::sync::{mpsc, oneshot};
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Origin};
use tracing::{debug, instrument, warn};

static DIST_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../app/dist");

/// Internal server state.
#[derive(Clone, Debug)]
pub struct State {
    db: db::Database,
    tx: mpsc::Sender<devices::Command>,
}

impl State {
    /// Create a new `State` obhject.
    ///
    /// Pass sender `tx` used to map API calls to device requests.
    pub async fn new(tx: mpsc::Sender<devices::Command>) -> Result<Self, AppError> {
        Ok(Self {
            db: db::Database::new().await?,
            tx,
        })
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
async fn get_state(Extension(state): Extension<State>) -> Result<Json<models::Device>, AppError> {
    let (resp, rx) = oneshot::channel();
    let command = devices::Command::Read { resp };
    let _ = state.tx.send(command).await;
    Ok(Json(rx.await??))
}

#[instrument(skip_all)]
async fn set_temperature(
    Json(payload): Json<models::TargetTemperature>,
    Extension(state): Extension<State>,
) -> Result<(), AppError> {
    debug!("Set temperature to {}", payload.target_temperature);

    let (resp, rx) = oneshot::channel();
    let command = devices::Command::SetTemperature {
        temperature: payload.target_temperature,
        resp,
    };
    let _ = state.tx.send(command).await;
    rx.await?
}

#[instrument(skip_all)]
async fn get_recipes(
    Extension(state): Extension<State>,
) -> Result<Json<models::Recipes>, AppError> {
    let recipes = state.db.recipes().await?;
    Ok(Json(recipes))
}

#[instrument(skip_all)]
async fn get_recipe(
    Path(id): Path<i64>,
    Extension(state): Extension<State>,
) -> Result<Json<models::Recipe>, AppError> {
    let recipe = state.db.recipe(id).await?;

    Ok(Json(recipe))
}

#[instrument(skip_all)]
async fn post_recipe(Json(payload): Json<models::NewRecipe>, Extension(state): Extension<State>) {
    debug!("Storing {:?}", payload);

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

/// Start the web server.
#[instrument]
pub async fn run(state: State) -> Result<(), AppError> {
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
        .route("/api/temperature", post(set_temperature))
        .route("/api/recipes", get(get_recipes).post(post_recipe))
        .route("/api/recipes/:id", get(get_recipe))
        .layer(cors)
        .layer(ServiceBuilder::new().layer(AddExtensionLayer::new(state)));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
