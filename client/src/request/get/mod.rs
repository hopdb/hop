mod get_boolean;
mod get_bytes;
mod get_float;
mod get_integer;
mod get_list;
mod get_map;
mod get_set;
mod get_string;

pub use self::{
    get_boolean::GetBoolean, get_bytes::GetBytes, get_float::GetFloat, get_integer::GetInteger,
    get_list::GetList, get_map::GetMap, get_set::GetSet, get_string::GetString,
};

use super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A Get request that hasn't been configured with a type of value to get.
///
/// This is an intermediary that allows you to cleanly set a value knowing its
/// type from just the method signature, and get back a value in the same type.
///
/// For example, if you call [`GetUnconfigured::bool`], then you will get back a
/// configured [`GetBoolean`] struct which you can `await`. This struct will
/// resolve to a boolean on success. If you call [`GetUnconfigured::int`], then
/// you will get back a [`GetInteger`] which will resolve to an integer when
/// `await`ed.
///
/// If you just want a raw engine value or don't know the type, `await` this
/// struct to resolve to a value on success.
///
/// # Examples
///
/// Get the key "foo" which is known to be a boolean:
///
/// ```
/// use hop::Client;
///
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::memory();
///
/// client.set("foo").bool(true).await?;
///
/// // we know that it will resolve to a boolean on success
/// assert_eq!(true, client.get("foo").bool().await?);
/// # Ok(()) }
/// ```
///
/// [`GetUnconfigured::bool`]: #method.bool
/// [`GetUnconfigured::int`]: #method.int
/// [`GetBoolean`]: struct.GetBoolean.html
/// [`GetInteger`]: struct.GetInteger.html
pub struct GetUnconfigured<'a, B: Backend, K: AsRef<[u8]> + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Value, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> GetUnconfigured<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }

    /// An alias for [`bool`].
    ///
    /// [`bool`]: #method.bool
    pub fn boolean(self) -> GetBoolean<'a, B, K> {
        self.bool()
    }

    /// Get a key as a boolean.
    ///
    /// The returned struct, when `await`ed, will resolve to a boolean on
    /// success.
    ///
    /// # Examples
    ///
    /// Get the key "foo" as a boolean:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    ///
    /// client.set("foo").bool(true).await?;
    ///
    /// client.get("foo").bool().await?;
    /// # Ok(()) }
    /// ```
    pub fn bool(self) -> GetBoolean<'a, B, K> {
        GetBoolean::new(self.backend.unwrap(), self.key.unwrap())
    }

    /// Get a key as some bytes.
    ///
    /// The returned struct, when `await`ed, will resolve to a `Vec<u8>` on
    /// success.
    ///
    /// # Examples
    ///
    /// Get the key "foo" as some bytes:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").bytes([1u8, 2, 3, 4, 5].as_ref()).await?;
    ///
    /// assert_eq!(5, client.get("foo").bytes().await?.len());
    /// # Ok(()) }
    /// ```
    pub fn bytes(self) -> GetBytes<'a, B, K> {
        GetBytes::new(self.backend.unwrap(), self.key.unwrap())
    }

    /// Get a key as a float.
    ///
    /// The returned struct, when `await`ed, will resolve to a float on success.
    ///
    /// # Examples
    ///
    /// Get the key "foo" as a float:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").float(1.23).await?;
    ///
    /// assert!(client.get("foo").float().await? > 1.0);
    /// # Ok(()) }
    /// ```
    pub fn float(self) -> GetFloat<'a, B, K> {
        GetFloat::new(self.backend.unwrap(), self.key.unwrap())
    }

    /// An alias for [`int`].
    ///
    /// [`int`]: #method.int
    pub fn integer(self) -> GetInteger<'a, B, K> {
        self.int()
    }

    /// Get a key as an integer.
    ///
    /// The returned struct, when `await`ed, will resolve to an integer on
    /// success.
    ///
    /// # Examples
    ///
    /// Get the key "foo" as an integer:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").int(123).await?;
    ///
    /// assert_eq!(123, client.get("foo").int().await?);
    /// # Ok(()) }
    /// ```
    pub fn int(self) -> GetInteger<'a, B, K> {
        GetInteger::new(self.backend.unwrap(), self.key.unwrap())
    }

    /// Get a key as an list.
    ///
    /// The returned struct, when `await`ed, will resolve to a list on success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the list:
    ///
    /// - "foo"
    /// - "bar"
    /// - "baz"
    ///
    /// Then, retrieve it as a list.
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").list([b"foo".to_vec(), b"bar".to_vec(), b"baz".to_vec()].as_ref()).await?;
    /// # Ok(()) }
    /// ```
    pub fn list(self) -> GetList<'a, B, K> {
        GetList::new(self.backend.unwrap(), self.key.unwrap())
    }

    pub fn map(self) -> GetMap<'a, B, K> {
        GetMap::new(self.backend.unwrap(), self.key.unwrap())
    }

    /// Get a key to an list.
    ///
    /// The returned struct, when `await`ed, will resolve to a list on success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the set:
    ///
    /// - "foo"
    /// - "bar"
    /// - "foo"
    ///
    /// Then, confirm that it has only 2 items, since there are duplicate
    /// "foo"s.
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").set([b"foo".to_vec(), b"bar".to_vec(), b"foo".to_vec()].to_vec()).await?;
    ///
    /// assert_eq!(2, client.get("foo").set().await?.len());
    /// # Ok(()) }
    /// ```
    pub fn set(self) -> GetSet<'a, B, K> {
        GetSet::new(self.backend.unwrap(), self.key.unwrap())
    }

    /// An alias for [`str`].
    ///
    /// [`str`]: #method.str
    #[inline]
    pub fn string(self) -> GetString<'a, B, K> {
        self.str()
    }

    /// Get a key as a string.
    ///
    /// The returned struct, when `await`ed, will resolve to a string on
    /// success.
    ///
    /// # Examples
    ///
    /// Get the key "foo" as a string:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").str("bar").await?;
    ///
    /// assert_eq!("bar", client.get("foo").str().await?);
    /// # Ok(()) }
    /// ```
    pub fn str(self) -> GetString<'a, B, K> {
        GetString::new(self.backend.unwrap(), self.key.unwrap())
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin + 'a> Future
    for GetUnconfigured<'a, B, K>
{
    type Output = Result<Value, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut
                .replace(Box::pin(async move { backend.get(key.as_ref()).await }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
