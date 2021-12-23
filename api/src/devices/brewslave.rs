use crate::devices::Device;
use models::State;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct Brewslave {
    client: comm::Comm,
}

impl Brewslave {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: comm::Comm::new()?,
        })
    }
}

#[async_trait::async_trait]
impl Device for Brewslave {
    /// Set up the serial connection and poll for new temperature, stirrer and heater values.
    #[instrument]
    async fn read(&self) -> anyhow::Result<State> {
        let state = self.client.read_state().await?;
        debug!("read {:?}", state);

        Ok(State {
            current_temperature: state.current_temperature,
            target_temperature: state.target_temperature,
            stirrer_on: state.stirrer_on,
            heater_on: state.heater_on,
            serial_problem: false,
        })
    }

    #[instrument]
    async fn set_temperature(&mut self, temperature: f32) -> anyhow::Result<()> {
        Ok(self.client.set_temperature(temperature).await?)
    }
}
