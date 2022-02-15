use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt::Display;
use std::str::FromStr;

/// Device state.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Device {
    /// Current temperature or `None` if sensor reading failed.
    pub current_temperature: Option<f32>,
    /// Target temperature or `None` if sensor reading failed.
    pub target_temperature: Option<f32>,
    pub stirrer_on: bool,
    pub heater_on: bool,
    pub serial_problem: bool,
}

/// Recipe step.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Step {
    pub description: String,
    pub target_temperature: f32,
    pub duration: std::time::Duration,
}

/// Recipe identifier newtype.
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RecipeId(pub i64);

impl Display for RecipeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RecipeId {
    type Err = <i64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>().map(Self)
    }
}

impl From<i64> for RecipeId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<RecipeId> for i64 {
    fn from(id: RecipeId) -> Self {
        id.0
    }
}

/// Single recipe consisting of name and steps.
/// TODO: add additional metadata not related to the brewing itself.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,
    pub description: String,
    pub steps: Vec<Step>,
}

/// A new recipe going to be stored in the database.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NewRecipe {
    pub name: String,
    pub description: String,
    pub steps: Vec<Step>,
}

/// Result identifier of the new recipe.
#[derive(Debug, Deserialize, Serialize)]
pub struct NewRecipeResponse {
    pub id: RecipeId,
}

/// A new recipe going to be stored in the database.
#[derive(Debug, Deserialize, Serialize)]
pub struct NewBrew {
    pub id: RecipeId,
}

/// Brew identifier newtype.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BrewId(i64);

impl From<i64> for BrewId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<BrewId> for i64 {
    fn from(id: BrewId) -> Self {
        id.0
    }
}

/// Result identifier of a new brew.
#[derive(Debug, Deserialize, Serialize)]
pub struct NewBrewResponse {
    pub id: BrewId,
}

/// Multiple recipes.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Recipes {
    pub recipes: Vec<Recipe>,
}

/// A new target temperature to set on the device
#[derive(Debug, Deserialize, Serialize)]
pub struct TargetTemperature {
    pub target_temperature: f32,
}
