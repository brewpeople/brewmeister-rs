use crate::devices::Device;
use crate::AppError;
use tracing::instrument;

#[derive(Debug)]
pub struct Mock {
    temperature: f32,
}

impl Mock {
    pub fn new() -> Self {
        Self { temperature: 20.0 }
    }
}

#[async_trait::async_trait]
impl Device for Mock {
    #[instrument]
    async fn read(&self) -> Result<models::Device, AppError> {
        Ok(models::Device {
            current_temperature: Some(self.temperature),
            target_temperature: Some(self.temperature),
            stirrer_on: false,
            heater_on: false,
            serial_problem: false,
        })
    }

    #[instrument]
    async fn set_temperature(&mut self, temperature: f32) -> Result<(), AppError> {
        self.temperature = temperature;
        Ok(())
    }
}
