use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `set` command that will resolve to a float when `await`ed.
///
/// This is returned by [`SetUnconfigured::float`].
///
/// [`SetUnconfigured::float`]: struct.SetUnconfigured.html#method.float
pub struct SetFloat<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, f64, B::Error>,
    key: Option<K>,
    value: Option<f64>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> SetFloat<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: f64) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future for SetFloat<'a, B, K> {
    type Output = Result<f64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let float = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let value = backend.set(key.as_ref(), Value::Float(float)).await?;

                match value {
                    Value::Float(float) => Ok(float),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
