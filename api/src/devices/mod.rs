use crate::Result;
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

pub mod brewslave;
pub mod mock;

/// An external device capable of reading current real and set temperature as well as allowing
/// setting a target temperature.
#[async_trait::async_trait]
pub trait Device {
    /// Read model state from the device.
    async fn read(&self) -> Result<models::Device>;

    /// Set target temperature.
    async fn set_temperature(&mut self, temperature: f32) -> Result<()>;
}

/// Used by the caller to get a result back from a command.
type Responder<T> = oneshot::Sender<Result<T>>;

/// Commands to send to the device channel.
pub enum Command {
    Read {
        resp: Responder<models::Device>,
    },
    SetTemperature {
        temperature: f32,
        resp: Responder<()>,
    },
}

/// Run handler task receiving commands via `rx` and forwards them to the `device`.
#[instrument]
pub async fn run<D>(mut device: D, mut rx: mpsc::Receiver<Command>) -> Result<()>
where
    D: Device + std::fmt::Debug,
{
    while let Some(command) = rx.recv().await {
        match command {
            Command::Read { resp } => {
                let _ = resp.send(device.read().await);
            }
            Command::SetTemperature { temperature, resp } => {
                let _ = resp.send(device.set_temperature(temperature).await);
            }
        }
    }

    Ok(())
}
