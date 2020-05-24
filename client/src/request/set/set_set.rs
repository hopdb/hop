use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    iter::FromIterator,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `set` command that will resolve to a set when `await`ed.
///
/// This is returned by [`SetUnconfigured::set`].
///
/// [`SetUnconfigured::set`]: struct.SetUnconfigured.html#method.set
pub struct SetSet<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<Vec<u8>>, B::Error>,
    key: Option<K>,
    value: Option<Vec<Vec<u8>>>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> SetSet<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: Vec<Vec<u8>>) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for SetSet<'a, B, K>
{
    type Output = Result<Vec<Vec<u8>>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let value = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend
                    .set(key, Value::Set(FromIterator::from_iter(value)))
                    .await?;

                match value {
                    Value::Set(set) => Ok(set.into_iter().collect()),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::SetSet;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SetSet<MemoryBackend, Vec<u8>>: Send);
}
