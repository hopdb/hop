#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

pub mod backend;
pub mod model;
pub mod request;

pub use hop_engine::state::{KeyType, Value};

use backend::{Backend, MemoryBackend};
use request::{append::AppendUnconfigured, get::GetUnconfigured, set::SetUnconfigured, *};
use std::sync::Arc;

/// A client for interfacing over Hop instances.
#[derive(Clone, Debug)]
pub struct Client<B: Backend> {
    backend: Arc<B>,
}

impl<B: Backend> Client<B> {
    fn backend(&self) -> Arc<B> {
        Arc::clone(&self.backend)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Client<backend::ServerBackend> {
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
    /// println!("Increment value: {}", client.increment("foo").int().await?);
    /// # Ok(()) }
    pub async fn connect(
        addrs: impl tokio::net::ToSocketAddrs,
    ) -> Result<Self, <backend::ServerBackend as Backend>::Error> {
        let backend = backend::ServerBackend::connect(addrs).await.unwrap();

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
    /// println!("Incremented value: {}", client.increment("foo").int().await?);
    /// println!("Incremented again: {}", client.increment("foo").int().await?);
    /// # Ok(()) }
    /// ```
    pub fn memory() -> Self {
        Self {
            backend: Arc::new(MemoryBackend::new()),
        }
    }
}

impl<B: Backend> Client<B> {
    /// Append to a key's value.
    ///
    /// The struct returned includes methods for appending to each key type.
    /// Bytes can not be appended to a string, for example.
    ///
    /// If the key does not exist then an error is returned.
    ///
    /// Returns the new value on success.
    ///
    /// # Examples
    ///
    /// Append to a string:
    ///
    /// ```rust
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hop::Client;
    ///
    /// let mut client = Client::memory();
    ///
    /// client.set("foo").str("this is").await?;
    ///
    /// assert_eq!("this is a string", client.append("foo").str(" a string").await?);
    /// # Ok(()) }
    /// ```
    pub fn append<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> AppendUnconfigured<B, K> {
        AppendUnconfigured::new(self.backend(), key)
    }

    /// Decrements a float or integer key by one.
    ///
    /// Returns the new value on success.
    ///
    /// If the key does not exist, an integer key is created with a value of 0
    /// and then decremented by 1, resulting in the value being -1.
    ///
    /// This is an `O(1)` time complexity operation.
    pub fn decrement<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> Decrement<'_, B, K> {
        Decrement::new(self.backend(), key)
    }

    /// Delete a key by its name if it exists.
    ///
    /// Returns the deleted key name on success as a confirmation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// assert_eq!(1, client.increment("foo").int().await?);
    /// assert!(client.delete("foo").await.is_ok());
    ///
    /// // since the key doesn't exist anymore, incrementing it again will
    /// // result in a value of 1 again
    /// assert_eq!(1, client.increment("foo").int().await?);
    /// # Ok(()) }
    /// ```
    pub fn delete<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> Delete<'_, B, K> {
        Delete::new(self.backend(), key)
    }

    /// Echos the provided content back at you.
    ///
    /// Returns the input content.
    pub fn echo<K: AsRef<[u8]> + Send + Unpin>(&self, content: K) -> Echo<'_, B, K> {
        Echo::new(self.backend(), content)
    }

    /// Check if one or more keys exist.
    ///
    /// Returns `true` if all of the keys exist, or `false` if at least one of
    /// them does not.
    ///
    /// Refer to the documentation for the [`Exists`] request for more
    /// information on how to use the request struct returned by this method.
    ///
    /// This is an `O(n)` time complexity operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// // "foo" doesn't exist
    /// assert!(!client.exists().key("foo").await?);
    /// client.increment("foo").await?;
    ///
    /// // and now it does
    /// assert!(client.exists().key("foo").await?);
    /// # Ok(()) }
    /// ```
    ///
    /// [`Exists`]: request/exists/struct.Exists.html
    pub fn exists(&self) -> Exists<B> {
        Exists::new(self.backend())
    }

    /// Get a key's value.
    ///
    /// The returned request struct, [`GetUnconfigured`] can be `await`ed or can
    /// be called with the type-specific methods if you know the type of the
    /// key. If you call one of these methods, the type that `await`ing the
    /// struct resolves to will be more fine-grained.
    ///
    /// Refer to [`GetUnconfigured`] for more information and available methods.
    ///
    /// # Examples
    ///
    /// Get the "foo" key as an integer, since we know it is one:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    ///
    /// client.set("foo").int(123).await?;
    ///
    /// assert_eq!(123, client.get("foo").int().await?);
    /// # Ok(()) }
    /// ```
    ///
    /// If we didn't know the type of the key, we could match on a generic Value
    /// enum:
    ///
    /// ```
    /// use hop::{Client, Value};
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    ///
    /// client.set("foo").string("this is a string").await?;
    ///
    /// match client.get("foo").await? {
    ///     Value::Boolean(boolean) => println!("it's a boolean: {}", boolean),
    ///     Value::Integer(int) => println!("it's an integer: {}", int),
    ///     Value::String(string) => println!("it's a string: {}", string),
    ///     _ => println!("it's something else"),
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// [`GetUnconfigured`]: request/get/struct.GetUnconfigured.html
    pub fn get<'a, K: AsRef<[u8]> + Send + Unpin + 'a>(&self, key: K) -> GetUnconfigured<'a, B, K> {
        GetUnconfigured::new(self.backend(), key)
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
    /// Increment a nonexistent key, which will set it to 0 and then increment
    /// it:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// println!("New value: {}", client.increment("foo").int().await?);
    /// # Ok(()) }
    /// ```
    ///
    /// Increment the key "foo" by the integer 2 and then 4:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// assert_eq!(2, client.increment("foo").int().by(2).await?);
    /// assert_eq!(6, client.increment("foo").int().by(4).await?);
    /// # Ok(()) }
    /// ```
    pub fn increment<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> Increment<'_, B, K> {
        Increment::new(self.backend(), key)
    }

    /// Check if one or more keys is a specified key type.
    ///
    /// Returns `true` if all of the keys both exist and are the specified key
    /// type, or `false` if at least one of the keys does not exist or is of a
    /// different key type.
    ///
    /// Refer to the documentation for the [`Is`] request for more
    /// information on how to use the request struct returned by this method.
    ///
    /// This is an `O(n)` time complexity operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::{Client, KeyType};
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    ///
    /// // make a key named "foo" that is an integer
    /// client.set("foo").int(123).await?;
    ///
    /// // check if "foo" is an integer, which it is:
    /// assert!(client.is(KeyType::Integer).key("foo").await?);
    ///
    /// // now make a key named "bar" that is a float
    /// client.set("bar").float(1.23).await?;
    ///
    /// // now if we check that both are an integer, we'll get back false, since
    /// // "foo" is an integer but "bar" is not
    /// assert!(!client.is(KeyType::Integer).keys(&["foo", "bar"])?.await?);
    /// # Ok(()) }
    /// ```
    ///
    /// [`Exists`]: request/exists/struct.Exists.html
    pub fn is(&self, key_type: KeyType) -> Is<B> {
        Is::new(self.backend(), key_type)
    }

    /// Retrieve the type of a key.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::{Client, KeyType};
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").int(123).await?;
    ///
    /// assert_eq!(KeyType::Integer, client.key_type("foo").await?);
    /// # Ok(()) }
    /// ```
    pub fn key_type<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> Type<'_, B, K> {
        Type::new(self.backend(), key)
    }

    /// Retrieve a list of the keys of a map.
    ///
    /// Returns the list of keys on success.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").map([(b"key".to_vec(), b"value".to_vec())].to_vec()).await?;
    ///
    /// assert_eq!([b"key".to_vec()].to_vec(), client.keys("foo").await?);
    /// # Ok(()) }
    /// ```
    pub fn keys<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> Keys<'_, B, K> {
        Keys::new(self.backend(), key)
    }

    /// Retrieve the length of a key's value.
    ///
    /// This command only works with bytes, lists, and strings by default. If
    /// you want to specify to only retrieve the length if the key is of a
    /// certain type, use the methods on the returned [`Length`] request struct.
    ///
    /// Returns the value's length on success.
    ///
    /// # Examples
    ///
    /// Get the length of a list key:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").list([b"foo".to_vec(), b"bar".to_vec()].to_vec()).await?;
    ///
    /// assert_eq!(2, client.length("foo").await?);
    /// # Ok(()) }
    /// ```
    ///
    /// Get the length of a key, but *only* if it is a string:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").string("this is a string").await?;
    ///
    /// assert_eq!(16, client.length("foo").str().await?);
    ///
    /// // if you try to get the length of a string key and specify list, it
    /// // will error:
    /// assert!(client.length("foo").list().await.is_err());
    /// # Ok(()) }
    /// ```
    ///
    /// [`Length`]: request/struct.Length.html
    pub fn length<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> Length<'_, B, K> {
        Length::new(self.backend(), key)
    }

    /// Rename a key to a new key name, if the new key name doesn't already
    /// exist.
    ///
    /// Returns the new key value on success as a confirmation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.increment("foo").int().await?;
    /// println!("New key name: {:?}", client.rename("foo", "bar").await?);
    /// println!("New incremented value: {}", client.increment("foo").int().await?);
    /// # Ok(()) }
    /// ```
    pub fn rename<F: AsRef<[u8]> + Send + Unpin, T: AsRef<[u8]> + Send + Unpin>(
        &self,
        from: F,
        to: T,
    ) -> Rename<'_, B, F, T> {
        Rename::new(self.backend(), from, to)
    }

    /// Set a key to a new value, overriding it regardless of whether it exists
    /// and its current key type.
    ///
    /// Returns the new value on success as confirmation.
    ///
    /// Refer to [`SetUnconfigured`] for more information and available methods.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the integer `123`, and then set "foo" to the string
    /// "bar".
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// assert_eq!(123, client.set("foo").int(123).await?);
    ///
    /// assert_eq!("bar", client.set("foo").string("bar").await?);
    /// # Ok(()) }
    /// ```
    ///
    /// [`SetUnconfigured`]: request/set/struct.SetUnconfigured.html
    pub fn set<K: AsRef<[u8]> + Send + Unpin>(&self, key: K) -> SetUnconfigured<B, K> {
        SetUnconfigured::new(self.backend(), key)
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

#[cfg(test)]
mod tests {
    use super::{backend::MemoryBackend, Client};
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(Client<MemoryBackend>: Debug, Send, Sync);
}
