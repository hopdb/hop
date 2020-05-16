use super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::KeyType;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Increment<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, i64, B::Error>,
    key: Option<K>,
    kind: Option<KeyType>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> Increment<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            key: Some(key),
            kind: None,
            fut: None,
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

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future
    for Increment<'a, B, K>
{
    type Output = Result<i64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let key = self.key.take().expect("key only taken once");
            let kind = self.kind.take();

            self.fut.replace(Box::pin(async move {
                backend.increment(key.as_ref(), kind).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
