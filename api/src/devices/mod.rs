use crate::State;

pub mod brewslave;
pub mod mock;

#[async_trait::async_trait]
pub trait Device {
    async fn communicate(&self, state: State) -> anyhow::Result<()>;
}
