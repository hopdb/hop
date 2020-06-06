use super::MaybeInFlightFuture;
use crate::Backend;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Rename<
    'a,
    B: Backend,
    F: AsRef<[u8]> + 'a + Send + Unpin,
    T: AsRef<[u8]> + 'a + Send + Unpin,
> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, Vec<u8>, B::Error>,
    from: Option<F>,
    to: Option<T>,
}

impl<'a, B: Backend, F: AsRef<[u8]> + 'a + Send + Unpin, T: AsRef<[u8]> + 'a + Send + Unpin>
    Rename<'a, B, F, T>
{
    pub(crate) fn new(backend: Arc<B>, from: F, to: T) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            from: Some(from),
            to: Some(to),
        }
    }
}

impl<
        'a,
        B: Backend + Send + Sync + 'static,
        F: AsRef<[u8]> + Send + Unpin,
        T: AsRef<[u8]> + Send + Unpin,
    > Future for Rename<'a, B, F, T>
{
    type Output = Result<Vec<u8>, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };
            let from = self.from.take().expect("from only taken once");
            let to = self.to.take().expect("to only taken once");

            self.fut.replace(Box::pin(async move {
                let from = from.as_ref();
                let to = to.as_ref();

                backend.rename(from, to).await
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Rename;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Rename<MemoryBackend, Vec<u8>, Vec<u8>>: Send);
}
