use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `set` command that will resolve to bytes when `await`ed.
///
/// This is returned by [`SetUnconfigured::bytes`].
///
/// [`SetUnconfigured::bytes`]: struct.SetUnconfigured.html#method.bytes
pub struct SetBytes<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<u8>, B::Error>,
    key: Option<K>,
    value: Option<Vec<u8>>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> SetBytes<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: Vec<u8>) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future for SetBytes<'a, B, K> {
    type Output = Result<Vec<u8>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let value = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let value = backend.set(key.as_ref(), Value::Bytes(value)).await?;

                match value {
                    Value::Bytes(bytes) => Ok(bytes),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
