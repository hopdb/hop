use super::super::MaybeInFlightFuture;
use crate::Backend;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `get` command that will resolve to a boolean when `await`ed.
///
/// This is returned by [`GetUnconfigured::bool`].
///
/// [`GetUnconfigured::bool`]: struct.GetUnconfigured.html#method.bool
pub struct GetBoolean<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, bool, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> GetBoolean<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future
    for GetBoolean<'a, B, K>
{
    type Output = Result<bool, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let value = backend.get(key.as_ref()).await?;

                match value {
                    Value::Boolean(bool) => Ok(bool),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
