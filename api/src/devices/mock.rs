use crate::devices::Device;
use crate::State;
use tracing::instrument;

#[derive(Debug)]
pub struct Mock {
    temperature: f32,
}

impl Mock {
    pub fn new() -> Self {
        Self {
            temperature: 20.0,
        }
    }
}

#[async_trait::async_trait]
impl Device for Mock {
    /// Set up the serial connection and poll for new temperature, stirrer and heater values.
    #[instrument]
    async fn communicate(&self, state: State) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            let mut state = state.inner.write().await;
            state.current_temperature = self.temperature;
            state.target_temperature = self.temperature;
            state.stirrer_on = false;
            state.heater_on = false;
            state.serial_problem = false;
        }
    }
}
