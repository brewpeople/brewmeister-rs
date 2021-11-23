use serde::{Deserialize, Serialize};

/// State shared between backend API and frontend app.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct State {
    pub current_temperature: f32,
    pub target_temperature: f32,
    pub stirrer_on: bool,
    pub heater_on: bool,
}
