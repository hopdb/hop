use super::{CommandConfigurationError, MaybeInFlightFuture};
use crate::Backend;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use hop_engine::state::KeyType;

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
pub struct Is<B: Backend> {
    backend: Arc<B>,
    key_type: KeyType,
}

impl<'a, B: Backend> Is<B> {
    pub(crate) fn new(backend: Arc<B>, key_type: KeyType) -> Self {
        Self { backend, key_type }
    }

    /// Check that a single key is the specified type.
    ///
    /// Refer to the [struct docs] for more information.
    ///
    /// [struct docs]: #main
    pub fn key<K: AsRef<[u8]> + 'a + Send + Unpin>(self, key: K) -> IsConfigured<'a, B, K> {
        let mut keys = Vec::new();
        keys.push(key);

        IsConfigured::new(self.backend, self.key_type, keys)
    }

    /// Check that one or more keys is the specified type.
    ///
    /// Refer to the [struct docs] for more information.
    ///
    /// # Errors
    ///
    /// Returns [`CommandConfigurationError::NoKeys`] if no keys were provided.
    ///
    /// Returns [`CommandConfigurationError::TooManyKeys`] if more than the
    /// argument limit (255) worth of keys were provided.
    ///
    /// [`CommandConfigurationError::NoKeys`]: enum.CommandConfigurationError.html#variant.NoKeys
    /// [`CommandConfigurationError::TooManyKeys`]: enum.CommandConfigurationError.html#variant.TooManyKeys
    /// [struct docs]: #main
    pub fn keys<K: AsRef<[u8]> + 'a + Send + Unpin>(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> Result<IsConfigured<'a, B, K>, CommandConfigurationError> {
        let keys: Vec<K> = keys.into_iter().collect();

        if keys.is_empty() {
            return Err(CommandConfigurationError::NoKeys);
        } else if keys.len() > u8::MAX as usize {
            return Err(CommandConfigurationError::TooManyKeys);
        }

        Ok(IsConfigured::new(self.backend, self.key_type, keys))
    }
}

/// A configured request to check if one or more keys exist.
pub struct IsConfigured<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, bool, B::Error>,
    key_type: KeyType,
    keys: Option<Vec<K>>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> IsConfigured<'a, B, K> {
    fn new(backend: Arc<B>, key_type: KeyType, keys: Vec<K>) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key_type,
            keys: Some(keys),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + 'a + Send + Unpin> Future
    for IsConfigured<'a, B, K>
{
    type Output = Result<bool, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let keys = self.keys.take().expect("keys only taken once");
            let key_type = self.key_type;

            self.fut
                .replace(Box::pin(async move { backend.is(key_type, keys).await }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{Is, IsConfigured};
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Is<MemoryBackend>: Send);
    assert_impl_all!(IsConfigured<MemoryBackend, Vec<u8>>: Send);
}
