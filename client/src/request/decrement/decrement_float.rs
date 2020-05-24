use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `get` command that will resolve to a float when `await`ed.
///
/// This is returned by [`GetUnconfigured::float`].
///
/// [`GetUnconfigured::float`]: struct.GetUnconfigured.html#method.float
pub struct DecrementFloat<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    amount: f64,
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, f64, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> DecrementFloat<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            amount: 1f64,
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }

    pub fn by(mut self, amount: f64) -> Self {
        self.amount = amount;

        self
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for DecrementFloat<'a, B, K>
{
    type Output = Result<f64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let amount = self.amount;
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.decrement_by(key, Value::Float(amount)).await?;

                match value {
                    Value::Float(float) => Ok(float),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::DecrementFloat;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(DecrementFloat<MemoryBackend, Vec<u8>>: Send);
}
