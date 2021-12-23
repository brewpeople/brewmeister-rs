use models::State;

pub mod brewslave;
pub mod mock;

#[async_trait::async_trait]
pub trait Device {
    async fn read(&self) -> anyhow::Result<State>;
}
