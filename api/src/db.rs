use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

#[derive(Clone, Debug, Default)]
pub struct Database {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    recipes: models::Recipes,
}

impl Database {
    /// Get all known recipes.
    #[instrument]
    pub async fn recipes(&self) -> models::Recipes {
        let inner = self.inner.read().await;
        inner.recipes.clone()
    }

    /// Add a recipe.
    #[instrument]
    pub async fn add_recipe(&self, recipe: models::Recipe) {
        info!("Add new recipe {}", recipe.name);
        let mut inner = self.inner.write().await;
        inner.recipes.recipes.push(recipe);
    }
}
