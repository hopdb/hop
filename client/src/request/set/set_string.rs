use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `set` command that will resolve to a string when `await`ed.
///
/// This is returned by [`SetUnconfigured::string`].
///
/// [`SetUnconfigured::string`]: struct.SetUnconfigured.html#method.string
pub struct SetString<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, String, B::Error>,
    key: Option<K>,
    value: Option<String>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> SetString<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K, value: String) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
            value: Some(value),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future
    for SetString<'a, B, K>
{
    type Output = Result<String, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");
            let value = self.value.take().expect("value only taken once");

            self.fut.replace(Box::pin(async move {
                let value = backend.set(key.as_ref(), Value::String(value)).await?;

                match value {
                    Value::String(string) => Ok(string),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
