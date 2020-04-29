pub mod backend;

use async_std::net::ToSocketAddrs;
use backend::{Backend, MemoryBackend, ServerBackend};

/// A client for interfacing over Hop instances.
pub struct Client<B: Backend> {
    backend: B,
}

impl Client<ServerBackend> {
    /// Connect to a server instance of Hop by address.
    ///
    /// # Examples
    ///
    /// Connect to an instance of Hop on port 14000 of localhost:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::connect("localhost:14000").await?;
    /// println!("Increment value: {}", client.increment_int("foo").await?);
    /// # Ok(()) }
    pub async fn connect(
        addrs: impl ToSocketAddrs,
    ) -> Result<Self, <ServerBackend as Backend>::Error> {
        let backend = ServerBackend::connect(addrs).await.unwrap();

        Ok(Self { backend })
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
    /// use hop::Client;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// println!("Incremented value: {}", client.increment_int("foo").await?);
    /// println!("Incremented again: {}", client.increment_int("foo").await?);
    /// # Ok(()) }
    /// ```
    pub fn memory() -> Self {
        Self {
            backend: MemoryBackend::new(),
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
    pub async fn decrement(&mut self, key: impl AsRef<[u8]>) -> Result<i64, B::Error> {
        self.backend.decrement(key.as_ref()).await
    }

    /// Echos the provided content back at you.
    ///
    /// Returns the input content.
    pub async fn echo(&mut self, content: impl AsRef<[u8]>) -> Result<Vec<u8>, B::Error> {
        self.backend.echo(content.as_ref()).await
    }

    /// Increments a float or integer key by one.
    ///
    /// Returns the new value on success.
    ///
    /// If the key does not exist, an integer key is created with a value of 0
    /// and then incremented by 1, resulting in the value being 1.
    ///
    /// This is an `O(1)` time complexity operation.
    pub async fn increment(&mut self, key: impl AsRef<[u8]>) -> Result<i64, B::Error> {
        self.backend.increment(key.as_ref()).await
    }
}
