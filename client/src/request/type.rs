use super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::KeyType;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Type<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, KeyType, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> Type<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future for Type<'a, B, K> {
    type Output = Result<KeyType, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(
                async move { backend.key_type(key.as_ref()).await },
            ));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
