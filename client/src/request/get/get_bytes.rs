use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `get` command that will resolve to bytes when `await`ed.
///
/// This is returned by [`GetUnconfigured::bool`].
///
/// [`GetUnconfigured::bool`]: struct.GetUnconfigured.html#method.bool
pub struct GetBytes<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<u8>, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> GetBytes<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for GetBytes<'a, B, K>
{
    type Output = Result<Vec<u8>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.get(key).await?;

                match value {
                    Value::Bytes(bytes) => Ok(bytes),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::GetBytes;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(GetBytes<MemoryBackend, Vec<u8>>: Send);
}
