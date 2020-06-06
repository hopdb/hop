use super::super::MaybeInFlightFuture;
use crate::Backend;
use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use hop_engine::state::Value;

/// A configured `get` command that will resolve to a float when `await`ed.
///
/// This is returned by [`GetUnconfigured::float`].
///
/// [`GetUnconfigured::float`]: struct.GetUnconfigured.html#method.float
pub struct DecrementInteger<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    amount: i64,
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, i64, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> DecrementInteger<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            amount: 1,
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }

    pub fn by(mut self, amount: i64) -> Self {
        self.amount = amount;

        self
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for DecrementInteger<'a, B, K>
{
    type Output = Result<i64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let amount = self.amount;
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.decrement_by(key, Value::Integer(amount)).await?;

                match value {
                    Value::Integer(int) => Ok(int),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::DecrementInteger;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(DecrementInteger<MemoryBackend, Vec<u8>>: Send);
}
