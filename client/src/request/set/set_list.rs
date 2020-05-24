use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `set` command that will resolve to a list when `await`ed.
///
/// This is returned by [`SetUnconfigured::list`].
///
/// [`SetUnconfigured::list`]: struct.SetUnconfigured.html#method.list
pub struct SetList<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<Vec<u8>>, B::Error>,
    key: Option<K>,
    value: Option<Vec<Vec<u8>>>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> SetList<'a, B, K> {
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
    for SetList<'a, B, K>
{
    type Output = Result<Vec<Vec<u8>>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let value = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.set(key, Value::List(value)).await?;

                match value {
                    Value::List(list) => Ok(list),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::SetList;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SetList<MemoryBackend, Vec<u8>>: Send);
}
