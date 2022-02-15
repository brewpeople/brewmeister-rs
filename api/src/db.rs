use crate::{AppError, Result};
use futures::future::try_join_all;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, FromRow};
use std::convert::From;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
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

#[derive(FromRow)]
pub struct Step {
    pub position: i64,
    pub target_temperature: f32,
    pub duration: i64,
}

#[derive(FromRow)]
pub struct Brew {
    pub recipe_id: i64,
}

impl From<Recipe> for models::Recipe {
    fn from(recipe: Recipe) -> Self {
        Self {
            id: recipe.id.into(),
            name: recipe.title,
            description: recipe.description,
            steps: vec![],
        }
    }
}

impl From<Step> for models::Step {
    fn from(step: Step) -> Self {
        Self {
            target_temperature: step.target_temperature,
            duration: std::time::Duration::from_secs(step.duration as u64),
        }
    }
}

impl Database {
    /// Create new database. Use the environment variable `DATABASE_URL` to point to a valid sqlite
    /// database file.
    pub async fn new(database: Option<String>) -> Result<Self> {
        let url = database.unwrap_or_else(|| "sqlite::memory:".to_string());

        info!("Connecting to {url}");

        let options = SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
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

        let steps = sqlx::query_as::<_, Step>("SELECT * from steps WHERE recipe_id = ?")
            .bind(id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|step| step.into())
            .collect::<Vec<models::Step>>();

        let recipe = models::Recipe {
            id: recipe.id.into(),
            name: recipe.title,
            description: recipe.description,
            steps,
        };

        Ok(recipe)
    }

    /// Add a recipe.
    #[instrument]
    pub async fn add_recipe(&self, recipe: models::NewRecipe) -> Result<models::NewRecipeResponse> {
        let result = sqlx::query("INSERT INTO recipes (title, description) VALUES (?, ?)")
            .bind(recipe.name)
            .bind(recipe.description)
            .execute(&self.pool)
            .await?;

        let id = result.last_insert_rowid();

        let futures = recipe
            .steps
            .into_iter()
            .enumerate()
            .map(|(pos, step)| async move {
                sqlx::query(
                    "INSERT INTO steps (recipe_id, position, target_temperature, duration) VALUES (?, ?, ?, ?)",
                )
                .bind(id)
                .bind(pos as i64)
                .bind(&step.target_temperature)
                .bind(step.duration.as_secs() as i64)
                .execute(&self.pool)
                .await?;

                Ok::<_, AppError>(())
            })
            .collect::<Vec<_>>();

        try_join_all(futures).await?;

        Ok(models::NewRecipeResponse { id: id.into() })
    }

    /// Add a brew.
    #[instrument]
    pub async fn add_brew(&self, brew: models::NewBrew) -> Result<models::NewBrewResponse> {
        let brew_id: i64 = brew.id.into();

        let id = sqlx::query("INSERT INTO brews (recipe_id) VALUES (?)")
            .bind(brew_id)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

        Ok(models::NewBrewResponse { id: id.into() })
    }

    /// Get recipe by id.
    #[instrument]
    pub async fn recipe_for_brew(&self, brew_id: i64) -> Result<models::Recipe> {
        let brew = sqlx::query_as::<_, Brew>("SELECT recipe_id from brews WHERE id = ?")
            .bind(brew_id)
            .fetch_one(&self.pool)
            .await?;

        self.recipe(brew.recipe_id).await
    }

    /// Add new sample.
    #[instrument]
    pub async fn add_sample(&self, brew_id: i64, temperature: f32) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?;

        sqlx::query(
            "INSERT INTO brew_measurements (brew_id, timestamp, brew_temperature) VALUES (?, ?, ?)",
        )
        .bind(brew_id)
        .bind(timestamp.as_secs() as i64)
        .bind(temperature)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
