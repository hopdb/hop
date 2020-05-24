use super::MaybeInFlightFuture;
use crate::Backend;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Delete<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<u8>, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> Delete<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for Delete<'a, B, K>
{
    type Output = Result<Vec<u8>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                backend.delete(key).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Delete;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Delete<MemoryBackend, Vec<u8>>: Send);
}
