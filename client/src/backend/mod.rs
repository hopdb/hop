pub mod memory;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

pub use self::memory::MemoryBackend;

#[cfg(not(target_arch = "wasm32"))]
pub use self::server::ServerBackend;

use crate::model::StatsData;
use async_trait::async_trait;
use hop_engine::state::{KeyType, Value};

#[async_trait]
pub trait Backend {
    type Error;

    async fn decrement(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>;

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn echo(&self, content: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error>;

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool, Self::Error>;

    async fn increment(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>;

    async fn is<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        key_type: KeyType,
        keys: T,
    ) -> Result<bool, Self::Error>;

    async fn key_type(&self, key: &[u8]) -> Result<KeyType, Self::Error>;

    async fn keys(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error>;

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn set<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value, Self::Error>;

    async fn stats(&self) -> Result<StatsData, Self::Error>;
}
