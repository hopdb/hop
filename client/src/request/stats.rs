use super::MaybeInFlightFuture;
use crate::{model::StatsData, Backend};
use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stats<'a, B: Backend> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, StatsData, B::Error>,
}

impl<'a, B: Backend> Stats<'a, B> {
    pub(crate) fn new(backend: Arc<B>) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
        }
    }
}

impl<'a, B: Backend + Send + 'static> Future for Stats<'a, B> {
    type Output = Result<StatsData, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = { self.backend.take().expect("backend only taken once") };

            self.fut
                .replace(Box::pin(async move { backend.stats().await }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Stats<MemoryBackend>: Send);
}
