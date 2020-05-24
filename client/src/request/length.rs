use super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::KeyType;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// Request to retrieve the length of a key, optionally only if it is of a
/// certain type.
pub struct Length<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, i64, B::Error>,
    key: Option<K>,
    kind: Option<KeyType>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> Length<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            key: Some(key),
            kind: None,
            fut: None,
        }
    }

    /// Retrieve the length *only* if the key is some bytes.
    pub fn bytes(mut self) -> Self {
        self.kind.replace(KeyType::Bytes);

        self
    }

    /// Retrieve the length *only* if the key is list.
    pub fn list(mut self) -> Self {
        self.kind.replace(KeyType::List);

        self
    }

    /// An alais for [`str`].
    ///
    /// [`str`]: #method.str
    pub fn string(self) -> Self {
        self.str()
    }

    /// Retrieve the length *only* if the key is a string.
    pub fn str(mut self) -> Self {
        self.kind.replace(KeyType::String);

        self
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for Length<'a, B, K>
{
    type Output = Result<i64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let key = self.key.take().expect("key only taken once");
            let kind = self.kind.take();

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                backend.length(key, kind).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Length;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Length<MemoryBackend, Vec<u8>>: Send);
}
