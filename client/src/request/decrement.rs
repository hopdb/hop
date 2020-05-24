use super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::KeyType;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Decrement<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, i64, B::Error>,
    key: Option<K>,
    kind: Option<KeyType>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> Decrement<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            kind: None,
        }
    }

    pub fn float(mut self) -> Self {
        self.kind.replace(KeyType::Float);

        self
    }

    pub fn int(mut self) -> Self {
        self.kind.replace(KeyType::Integer);

        self
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for Decrement<'a, B, K>
{
    type Output = Result<i64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let kind = self.kind.take();

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                backend.decrement(key, kind).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Decrement;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Decrement<MemoryBackend, Vec<u8>>: Send);
}
