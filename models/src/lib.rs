use serde::{Deserialize, Serialize};

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

/// Single recipe consisting of name and steps.
/// TODO: add additional metadata not related to the brewing itself.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Recipe {
    pub id: i64,
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
