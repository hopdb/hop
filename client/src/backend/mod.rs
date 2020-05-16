mod memory;
mod server;

pub use self::{memory::MemoryBackend, server::ServerBackend};

use crate::model::StatsData;
use async_trait::async_trait;
use hop_engine::state::KeyType;

#[async_trait]
pub trait Backend {
    type Error;

    async fn decrement(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>;

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn increment(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>;

    async fn stats(&self) -> Result<StatsData, Self::Error>;
}
