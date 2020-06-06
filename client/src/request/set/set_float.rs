use super::super::MaybeInFlightFuture;
use crate::Backend;
use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use hop_engine::state::Value;

/// A configured `set` command that will resolve to a float when `await`ed.
///
/// This is returned by [`SetUnconfigured::float`].
///
/// [`SetUnconfigured::float`]: struct.SetUnconfigured.html#method.float
pub struct SetFloat<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, f64, B::Error>,
    key: Option<K>,
    value: Option<f64>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> SetFloat<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: f64) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for SetFloat<'a, B, K>
{
    type Output = Result<f64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let float = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.set(key, Value::Float(float)).await?;

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
    use super::SetFloat;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SetFloat<MemoryBackend, Vec<u8>>: Send);
}
