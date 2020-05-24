mod increment_float;
mod increment_int;

pub use self::{increment_float::IncrementFloat, increment_int::IncrementInteger};

use super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::{KeyType, Value};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Increment<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Value, B::Error>,
    key: Option<K>,
    kind: Option<KeyType>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> Increment<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            kind: None,
        }
    }

    pub fn float(self) -> IncrementFloat<'a, B, K> {
        IncrementFloat::new(self.backend.unwrap(), self.key.unwrap())
    }

    pub fn int(self) -> IncrementInteger<'a, B, K> {
        IncrementInteger::new(self.backend.unwrap(), self.key.unwrap())
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for Increment<'a, B, K>
{
    type Output = Result<Value, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let kind = self.kind.take();

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();

                backend.increment(key, kind).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Increment;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Increment<MemoryBackend, Vec<u8>>: Send);
}
