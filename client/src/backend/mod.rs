mod memory;
mod server;

pub use self::{
    memory::MemoryBackend,
    server::ServerBackend,
};

use async_trait::async_trait;

#[async_trait]
pub trait Backend {
    type Error;

    async fn decrement_int(&mut self, key: &[u8]) -> Result<i64, Self::Error>;

    async fn increment(&mut self, key: &[u8]) -> Result<i64, Self::Error>;

    async fn increment_int(&mut self, key: &[u8]) -> Result<i64, Self::Error>;
}
