use super::super::MaybeInFlightFuture;
use crate::Backend;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use dashmap::DashMap;
use hop_engine::state::Value;

/// A configured `set` command that will resolve to a map when `await`ed.
///
/// This is returned by [`SetUnconfigured::map`].
///
/// [`SetUnconfigured::map`]: struct.SetUnconfigured.html#method.map
pub struct SetMap<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, DashMap<Vec<u8>, Vec<u8>>, B::Error>,
    key: Option<K>,
    value: Option<DashMap<Vec<u8>, Vec<u8>>>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> SetMap<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: DashMap<Vec<u8>, Vec<u8>>) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for SetMap<'a, B, K>
{
    type Output = Result<DashMap<Vec<u8>, Vec<u8>>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let value = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.set(key, Value::Map(value)).await?;

                match value {
                    Value::Map(map) => Ok(map),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::SetMap;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SetMap<MemoryBackend, Vec<u8>>: Send);
}
