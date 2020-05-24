use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `set` command that will resolve to an integer when `await`ed.
///
/// This is returned by [`SetUnconfigured::int`].
///
/// [`SetUnconfigured::int`]: struct.SetUnconfigured.html#method.int
pub struct SetInteger<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, i64, B::Error>,
    key: Option<K>,
    value: Option<i64>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> SetInteger<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: i64) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for SetInteger<'a, B, K>
{
    type Output = Result<i64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let int = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.set(key, Value::Integer(int)).await?;

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
    use super::SetInteger;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SetInteger<MemoryBackend, Vec<u8>>: Send);
}
