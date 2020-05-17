use super::MaybeInFlightFuture;
use crate::Backend;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

#[derive(Clone, Debug)]
pub enum ExistsError {
    NoKeys,
    TooManyKeys,
}

impl Display for ExistsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoKeys => f.write_str("no keys were provided"),
            Self::TooManyKeys => f.write_str("too many keys were provided"),
        }
    }
}

impl Error for ExistsError {}

/// Request to determine whether one or more keys exist.
///
/// Without creating a trait, it's not possible using the standard library to
/// cleanly represent "this argument can either be one argument or an iterator
/// of argument." Because of this, [`Client::exists`] returns this intermediary
/// struct which contains two methods, [`key`] and [`keys`].
///
/// After using one of these methods, you'll get back a separate "finalised"
/// request struct, which is ready to be `await`ed with your key arguments.
///
/// # Examples
///
/// Check if one, and then two, keys exist:
///
/// ```
/// use hop::Client;
///
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::memory();
/// // make a "foo" key
/// client.increment("foo").await?;
/// // and now check that it exists
/// assert!(client.exists().key("foo").await?);
///
/// // now check if "foo" *and* "bar" exist - clearly "bar" doesn't exist, so
/// // this will be false.
/// //
/// // additionally, `keys` can error if you provide 0 or more than 255
/// // arguments
/// assert!(!client.exists().keys(&["foo", "bar"])?.await?);
/// # Ok(()) }
/// ```
///
/// [`Client::exists`]: ../../struct.Client.html#method.exists
/// [`key`]: #method.key
/// [`keys`]: #method.keys
pub struct Exists<B: Backend> {
    backend: Arc<B>,
}

impl<'a, B: Backend> Exists<B> {
    pub(crate) fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }

    /// Check that a single key exists.
    ///
    /// Refer to the [struct docs] for more information.
    ///
    /// [struct docs]: #main
    pub fn key<K: AsRef<[u8]> + 'a + Unpin + Send + Sync>(
        self,
        key: K,
    ) -> ExistsConfigured<'a, B, K> {
        let mut keys = Vec::new();
        keys.push(key);

        ExistsConfigured::new(self.backend, keys)
    }

    /// Check that one or more keys exist.
    ///
    /// Refer to the [struct docs] for more information.
    ///
    /// # Errors
    ///
    /// Returns [`ExistsError::NoKeys`] if no keys were provided.
    ///
    /// Returns [`ExistsError::TooManyKeys`] if more than the argument limit
    /// (255) worth of keys were provided.
    ///
    /// [`ExistsError::NoKeys`]: enum.ExistsError.html#variant.NoKeys
    /// [`ExistsError::TooManyKeys`]: enum.ExistsError.html#variant.TooManyKeys
    /// [struct docs]: #main
    pub fn keys<K: AsRef<[u8]> + 'a + Unpin + Send + Sync>(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> Result<ExistsConfigured<'a, B, K>, ExistsError> {
        let keys: Vec<K> = keys.into_iter().collect();

        if keys.is_empty() {
            return Err(ExistsError::NoKeys);
        } else if keys.len() > u8::MAX as usize {
            return Err(ExistsError::TooManyKeys);
        }

        Ok(ExistsConfigured::new(self.backend, keys))
    }
}

/// A configured request to check if one or more keys exist.
pub struct ExistsConfigured<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin + Send + Sync> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, bool, B::Error>,
    keys: Option<Vec<K>>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin + Send + Sync> ExistsConfigured<'a, B, K> {
    fn new(backend: Arc<B>, keys: Vec<K>) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            keys: Some(keys),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + 'a + Unpin + Send + Sync> Future
    for ExistsConfigured<'a, B, K>
{
    type Output = Result<bool, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let keys = self.keys.take().expect("keys only taken once");

            self.fut.replace(Box::pin(
                async move { backend.exists((*keys).iter()).await },
            ));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
