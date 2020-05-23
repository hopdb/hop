use super::super::MaybeInFlightFuture;
use crate::Backend;
use dashmap::DashMap;
use hop_engine::state::Value;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// A configured `get` command that will resolve to a map when `await`ed.
///
/// This is returned by [`GetUnconfigured::map`].
///
/// [`GetUnconfigured::map`]: struct.GetUnconfigured.html#method.map
pub struct GetMap<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, DashMap<Vec<u8>, Vec<u8>>, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Unpin> GetMap<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Unpin> Future for GetMap<'a, B, K> {
    type Output = Result<DashMap<Vec<u8>, Vec<u8>>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let value = backend.get(key.as_ref()).await?;

                match value {
                    Value::Map(map) => Ok(map),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
