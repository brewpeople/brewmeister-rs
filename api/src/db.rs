use crate::{AppError, Result};
use futures::future::try_join_all;
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

#[derive(FromRow)]
pub struct Step {
    pub description: String,
    pub target_temperature: i64,
    pub duration: i64,
}

#[derive(FromRow)]
pub struct Brew {
    pub recipe_id: i64,
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

impl From<Step> for models::Step {
    fn from(step: Step) -> Self {
        Self {
            description: step.description,
            target_temperature: step.target_temperature as f32,
            duration: std::time::Duration::from_secs(step.duration as u64),
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

        let steps = sqlx::query_as::<_, Step>("SELECT * from steps INNER JOIN recipe_steps ON recipe_steps.step_id = steps.id WHERE recipe_steps.recipe_id = ?")
            .bind(id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|step| step.into())
            .collect::<Vec<models::Step>>();

        let recipe = models::Recipe {
            id: recipe.id,
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

        // TODO: check which steps already exist
        let futures = recipe.steps.iter().map(|step| async {
            let result = sqlx::query("INSERT INTO steps (description, target_temperature, duration) VALUES (?, ?, ?)")
                .bind(&step.description)
                .bind(&step.target_temperature)
                .bind(step.duration.as_secs() as i64)
                .execute(&self.pool).await?;

            Ok::<_, AppError>(result.last_insert_rowid())
        }).collect::<Vec<_>>();

        for (pos, step_id) in try_join_all(futures).await?.iter().enumerate() {
            sqlx::query("INSERT INTO recipe_steps (recipe_id, step_id, position) VALUES (?, ?, ?)")
                .bind(id)
                .bind(step_id)
                .bind(pos as i64)
                .execute(&self.pool)
                .await?;
        }

        Ok(models::NewRecipeResponse { id })
    }

    /// Add a brew.
    #[instrument]
    pub async fn add_brew(&self, brew: models::NewBrew) -> Result<models::NewBrewResponse> {
        let id = sqlx::query("INSERT INTO brews (recipe_id) VALUES (?)")
            .bind(brew.recipe_id)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

        Ok(models::NewBrewResponse { id })
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
}
