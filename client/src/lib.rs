#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

pub mod backend;
pub mod model;
pub mod request;

use backend::{Backend, MemoryBackend, ServerBackend};
use request::*;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;

/// A client for interfacing over Hop instances.
pub struct Client<B: Backend> {
    backend: Arc<B>,
}

impl<B: Backend> Client<B> {
    fn backend(&self) -> Arc<B> {
        Arc::clone(&self.backend)
    }
}

impl Client<ServerBackend> {
    /// Connect to a server instance of Hop by address.
    ///
    /// # Examples
    ///
    /// Connect to an instance of Hop on port 14000 of localhost:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hop::Client;
    ///
    /// let mut client = Client::connect("localhost:14000").await?;
    /// println!("Increment value: {}", client.increment("foo").await?);
    /// # Ok(()) }
    pub async fn connect(
        addrs: impl ToSocketAddrs,
    ) -> Result<Self, <ServerBackend as Backend>::Error> {
        let backend = ServerBackend::connect(addrs).await.unwrap();

        Ok(Self {
            backend: Arc::new(backend),
        })
    }
}

impl Client<MemoryBackend> {
    /// Create a local memory-backend Hop instance.
    ///
    /// This is similar to opening an in-memory SQLite instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hop::Client;
    ///
    /// let mut client = Client::memory();
    /// println!("Incremented value: {}", client.increment("foo").await?);
    /// println!("Incremented again: {}", client.increment("foo").await?);
    /// # Ok(()) }
    /// ```
    pub fn memory() -> Self {
        Self {
            backend: Arc::new(MemoryBackend::new()),
        }
    }
}

impl<B: Backend> Client<B> {
    /// Decrements a float or integer key by one.
    ///
    /// Returns the new value on success.
    ///
    /// If the key does not exist, an integer key is created with a value of 0
    /// and then decremented by 1, resulting in the value being -1.
    ///
    /// This is an `O(1)` time complexity operation.
    pub fn decrement<K: AsRef<[u8]> + Unpin>(&self, key: K) -> Decrement<'_, B, K> {
        Decrement::new(self.backend(), key)
    }

    /// Echos the provided content back at you.
    ///
    /// Returns the input content.
    pub fn echo<K: AsRef<[u8]> + Unpin>(&self, content: K) -> Echo<'_, B, K> {
        Echo::new(self.backend(), content)
    }

    /// Increments a float or integer key by one.
    ///
    /// Returns the new value on success.
    ///
    /// If the key does not exist, an integer key is created with a value of 0
    /// and then incremented by 1, resulting in the value being 1.
    ///
    /// This is an `O(1)` time complexity operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// println!("New value: {}", client.increment("foo").await?);
    /// # Ok(()) }
    /// ```
    pub fn increment<K: AsRef<[u8]> + Unpin>(&self, key: K) -> Increment<'_, B, K> {
        Increment::new(self.backend(), key)
    }

    /// Retrieve statistics about the current runtime of Hop.
    ///
    /// When Hop is restarted, many of the statistics - like commands run - are
    /// reset.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// let stats = client.stats().await?;
    /// println!("Successful commands: {}", stats.commands_successful());
    /// # Ok(()) }
    /// ```
    pub fn stats(&self) -> Stats<'_, B> {
        Stats::new(self.backend())
    }
}
