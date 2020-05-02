mod memory;
mod server;

pub use self::{memory::MemoryBackend, server::ServerBackend};

use async_trait::async_trait;

#[async_trait]
pub trait Backend {
    type Error;

    async fn decrement(&self, key: &[u8]) -> Result<i64, Self::Error>;

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn increment(&self, key: &[u8]) -> Result<i64, Self::Error>;
}
