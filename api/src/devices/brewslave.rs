use crate::devices::Device;
use crate::State;
use tracing::{debug, error, instrument};

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
    async fn communicate(&self, state: State) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            let mut state = state.inner.write().await;

            match self.client.read_state().await {
                Ok(current) => {
                    // We could impl `From<comm::State>` in `models` however that pulls in comm into
                    // the `app` frontend which makes targetting wasm32-unknown-unknown a bit painful.
                    state.current_temperature = current.current_temperature;
                    state.target_temperature = current.target_temperature;
                    state.stirrer_on = current.stirrer_on;
                    state.heater_on = current.heater_on;
                    state.serial_problem = false;
                    debug!("read {:?}", state);
                }
                Err(err) => {
                    error!("{}", err);
                    state.serial_problem = true;
                }
            }
        }
    }
}
