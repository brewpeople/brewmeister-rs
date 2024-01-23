use crate::{db, devices, program, AppError, Result};
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum_extra::headers::HeaderMap;
use axum_extra::routing::{RouterExt, TypedPath};
use http::HeaderValue;
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use tokio::sync::oneshot;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, instrument, warn};

static DIST_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../app/dist");

/// Internal server state.
#[derive(Clone, Debug)]
pub struct AppState {
    db: db::Database,
    device_tx: devices::Sender,
    brew_tx: program::Sender,
}

impl AppState {
    /// Create a new `State` obhject.
    ///
    /// Pass sender `tx` used to map API calls to device requests.
    pub async fn new(
        db: db::Database,
        device_tx: devices::Sender,
        brew_tx: program::Sender,
    ) -> Result<Self> {
        Ok(Self {
            db,
            device_tx,
            brew_tx,
        })
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self),
        )
            .into_response()
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/state")]
struct StateRoute;

#[instrument]
async fn get_state(_: StateRoute, State(state): State<AppState>) -> Result<Json<models::Device>> {
    let (resp, rx) = oneshot::channel();
    let command = devices::Command::Read { resp };
    let _ = state.device_tx.send(command).await;
    Ok(Json(rx.await??))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/recipes")]
struct RecipesRoute;

#[instrument(skip_all)]
async fn get_recipes(
    _: RecipesRoute,
    State(state): State<AppState>,
) -> Result<Json<models::Recipes>> {
    let recipes = state.db.recipes().await?;
    Ok(Json(recipes))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/recipes/:id")]
struct RecipeRoute {
    id: models::RecipeId,
}

#[instrument(skip_all)]
async fn get_recipe(
    RecipeRoute { id }: RecipeRoute,
    State(state): State<AppState>,
) -> Result<Json<models::Recipe>> {
    let recipe = state.db.recipe(id).await?;

    Ok(Json(recipe))
}

#[instrument(skip_all)]
async fn post_recipe(
    _: RecipesRoute,
    State(state): State<AppState>,
    Json(payload): Json<models::NewRecipe>,
) -> Result<Json<models::NewRecipeResponse>> {
    debug!("Storing {:?}", payload);

    let result = state.db.add_recipe(payload).await?;
    Ok(Json(result))
}

#[derive(TypedPath)]
#[typed_path("/api/brews")]
struct BrewsRoute;

#[instrument(skip(state))]
async fn start_brew(
    _: BrewsRoute,
    State(state): State<AppState>,
    Json(payload): Json<models::NewBrew>,
) -> Result<()> {
    debug!("Start brew");

    let recipe = state.db.recipe(payload.id).await?;
    let result = state.db.add_brew(recipe.id).await?;
    let (resp, rx) = oneshot::channel();

    let command = program::Command::Start {
        id: result.id,
        steps: recipe.steps,
        resp,
    };

    let _ = state.brew_tx.send(command).await;
    rx.await?
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/:path")]
struct StaticFileRoute {
    path: String,
}

#[instrument]
async fn get_static(StaticFileRoute { path }: StaticFileRoute) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();

    match DIST_DIR.get_file(&path) {
        Some(file) => {
            if let Some(e) = file.path().extension().and_then(|e| e.to_str()) {
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/")]
struct IndexRoute;

#[instrument]
async fn get_index(_: IndexRoute) -> (StatusCode, HeaderMap, Vec<u8>) {
    get_static(StaticFileRoute {
        path: "index.html".to_string(),
    })
    .await
}

/// Start the web server.
#[instrument]
pub async fn run(state: AppState) -> Result<()> {
    // Only useful if we run the app via `trunk serve`, if not we serve the static files directly.
    let cors = CorsLayer::new()
        .allow_origin("http://0.0.0.0:8080".parse::<HeaderValue>()?)
        .allow_methods(vec![Method::GET, Method::POST]);

    let trace = TraceLayer::new_for_http();

    let compression = CompressionLayer::new().gzip(true).deflate(true);

    let app = Router::new()
        .typed_get(get_index)
        .typed_get(get_static)
        .typed_post(start_brew)
        .typed_get(get_recipes)
        .typed_post(post_recipe)
        .typed_get(get_recipe)
        .typed_get(get_state)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(compression)
                .layer(trace)
                .layer(cors),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen to ctrl-c");
        })
        .await?;

    Ok(())
}
