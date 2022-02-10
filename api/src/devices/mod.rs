pub mod brewslave;
pub mod mock;

use crate::{AppError, State};
use tracing::{error, instrument};

/// An external device capable of reading current real and set temperature as well as allowing
/// setting a target temperature.
#[async_trait::async_trait]
pub trait Device {
    /// Read model state from the device.
    async fn read(&self) -> Result<models::Device, AppError>;

    /// Set target temperature.
    async fn set_temperature(&mut self, temperature: f32) -> Result<(), AppError>;

    /// Start polling `device` and updating `state`.
    #[instrument]
    async fn run(&self, state: State) -> Result<(), AppError>
    where
        Self: std::fmt::Debug,
    {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            match self.read().await {
                Ok(new) => {
                    let mut state = state.device.write().await;
                    state.current_temperature = new.current_temperature;
                    state.target_temperature = new.current_temperature;
                    state.stirrer_on = new.stirrer_on;
                    state.heater_on = new.heater_on;
                    state.serial_problem = false;
                }
                Err(err) => {
                    error!("Error reading from device: {err}");

                    let mut state = state.device.write().await;
                    state.serial_problem = true;
                }
            }
        }
    }
}
