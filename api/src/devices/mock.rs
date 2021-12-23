use crate::devices::Device;
use models::State;
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
    async fn read(&self) -> anyhow::Result<State> {
        Ok(State {
            current_temperature: self.temperature,
            target_temperature: self.temperature,
            stirrer_on: false,
            heater_on: false,
            serial_problem: false,
        })
    }

    #[instrument]
    async fn set_temperature(&mut self, temperature: f32) -> anyhow::Result<()> {
        self.temperature = temperature;
        Ok(())
    }
}
