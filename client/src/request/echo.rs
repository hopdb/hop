use super::MaybeInFlightFuture;
use crate::Backend;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Echo<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    content: Option<K>,
    fut: MaybeInFlightFuture<'a, Vec<Vec<u8>>, B::Error>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> Echo<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, content: K) -> Self {
        Self {
            backend: Some(backend),
            content: Some(content),
            fut: None,
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for Echo<'a, B, K>
{
    type Output = Result<Vec<Vec<u8>>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let content = self.content.take().expect("content only taken once");

            self.fut.replace(Box::pin(async move {
                let content = content.as_ref();
                backend.echo(content).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Echo;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Echo<MemoryBackend, Vec<u8>>: Send);
}
