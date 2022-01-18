use crate::AppError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

#[derive(Clone, Debug)]
pub struct Database {
    path: PathBuf,
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Inner {
    recipes: models::Recipes,
}

impl Database {
    /// Create new database with the given path pointing to the serialized JSON file.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let path = path.as_ref().to_path_buf();

        let inner = if path.exists() {
            serde_json::from_reader(BufReader::new(File::open(&path)?))?
        } else {
            Inner::default()
        };

        Ok(Self {
            path,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    /// Get all known recipes.
    #[instrument]
    pub async fn recipes(&self) -> models::Recipes {
        let inner = self.inner.read().await;
        inner.recipes.clone()
    }

    /// Add a recipe.
    #[instrument]
    pub async fn add_recipe(&self, recipe: models::Recipe) -> Result<(), AppError> {
        info!("Add new recipe {}", recipe.name);
        let mut inner = self.inner.write().await;
        inner.recipes.recipes.push(recipe);

        info!("Writing {:?}", self.path);
        serde_json::to_writer(BufWriter::new(File::create(&self.path)?), &(*inner))?;
        Ok(())
    }
}
