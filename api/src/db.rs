use crate::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, FromRow};
use std::convert::From;
use std::env;
use std::str::FromStr;
use tracing::{info, instrument};

#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(FromRow)]
struct Recipe {
    pub id: i64,
    pub title: String,
    pub description: String,
}

impl From<Recipe> for models::Recipe {
    fn from(recipe: Recipe) -> Self {
        Self {
            id: recipe.id,
            name: recipe.title,
            description: recipe.description,
            steps: vec![],
        }
    }
}

impl Database {
    /// Create new database. Use the environment variable `DATABASE_URL` to point to a valid sqlite
    /// database file.
    pub async fn new() -> Result<Self> {
        let url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

        info!("Connecting to {url}");

        let options = SqliteConnectOptions::from_str(&url)?
            .disable_statement_logging()
            .to_owned();

        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        sqlx::query(include_str!("sql/create.sql"))
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    /// Get all known recipes.
    #[instrument]
    pub async fn recipes(&self) -> Result<models::Recipes> {
        let recipes = sqlx::query_as::<_, Recipe>("SELECT * FROM recipes")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<models::Recipe>>();

        Ok(models::Recipes { recipes })
    }

    /// Get recipe by id.
    #[instrument]
    pub async fn recipe(&self, id: i64) -> Result<models::Recipe> {
        let recipe = sqlx::query_as::<_, Recipe>("SELECT * FROM recipes WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(recipe.into())
    }

    /// Add a recipe.
    #[instrument]
    pub async fn add_recipe(&self, recipe: models::NewRecipe) -> Result<()> {
        sqlx::query("INSERT INTO recipes (title, description) VALUES (?, ?)")
            .bind(recipe.name)
            .bind(recipe.description)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
