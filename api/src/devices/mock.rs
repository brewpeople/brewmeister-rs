use crate::devices::Device;
use crate::Result;
use std::time::Instant;
use tracing::instrument;

#[derive(Debug)]
pub struct Mock {
    target_temperature: f32,
    last_time: Instant,
    last_temperature: f32,
}

impl Mock {
    pub fn new() -> Self {
        Self {
            target_temperature: 20.0,
            last_time: Instant::now(),
            last_temperature: 19.0,
        }
    }

    fn current_temperature(&self) -> f32 {
        let duration = Instant::now() - self.last_time;
        let delta_temp = self.target_temperature - self.last_temperature;
        let y = (duration.as_secs_f32() / 5.0).tanh();
        self.last_temperature + y * delta_temp
    }
}

#[async_trait::async_trait]
impl Device for Mock {
    #[instrument]
    async fn read(&self) -> Result<models::Device> {
        Ok(models::Device {
            current_temperature: Some(self.current_temperature()),
            target_temperature: Some(self.target_temperature),
            stirrer_on: false,
            heater_on: false,
            serial_problem: false,
        })
    }

    #[instrument]
    async fn set_temperature(&mut self, temperature: f32) -> Result<()> {
        self.last_time = Instant::now();
        self.last_temperature = self.current_temperature();
        self.target_temperature = temperature;
        Ok(())
    }
}
