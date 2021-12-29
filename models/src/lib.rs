use serde::{Deserialize, Serialize};

/// Device state.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Device {
    pub current_temperature: f32,
    pub target_temperature: f32,
    pub stirrer_on: bool,
    pub heater_on: bool,
    pub serial_problem: bool,
}

/// Recipe step.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Step {
    pub description: String,
    pub target_temperature: f32,
    pub duration: std::time::Duration,
}

/// Single recipe consisting of name and steps.
/// TODO: add additional metadata not related to the brewing itself.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub steps: Vec<Step>,
}

/// Multiple recipes.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Recipes {
    pub recipes: Vec<Recipe>,
}
