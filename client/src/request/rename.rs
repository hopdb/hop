use super::MaybeInFlightFuture;
use crate::Backend;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Rename<'a, B: Backend, F: AsRef<[u8]> + 'a + Unpin, T: AsRef<[u8]> + 'a + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<u8>, B::Error>,
    from: Option<F>,
    to: Option<T>,
}

impl<'a, B: Backend, F: AsRef<[u8]> + 'a + Unpin, T: AsRef<[u8]> + 'a + Unpin> Rename<'a, B, F, T> {
    pub(crate) fn new(backend: Arc<B>, from: F, to: T) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            from: Some(from),
            to: Some(to),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, F: AsRef<[u8]> + Unpin, T: AsRef<[u8]> + Unpin> Future
    for Rename<'a, B, F, T>
{
    type Output = Result<Vec<u8>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let from = self.from.take().expect("from only taken once");
            let to = self.to.take().expect("to only taken once");

            self.fut.replace(Box::pin(async move {
                backend.rename(from.as_ref(), to.as_ref()).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}
