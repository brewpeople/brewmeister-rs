use models::State;

pub mod brewslave;
pub mod mock;

/// An external device capable of reading current real and set temperature as well as allowing
/// setting a target temperature.
#[async_trait::async_trait]
pub trait Device {
    /// Read model state from the device.
    async fn read(&self) -> anyhow::Result<State>;

    /// Set target temperature.
    async fn set_temperature(&mut self, temperature: f32) -> anyhow::Result<()>;
}
