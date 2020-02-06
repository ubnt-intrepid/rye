use crate::{
    section::{Section, Sections},
    tls::Guard,
};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::pin::Pin;

/// Run a test case asynchronously.
pub async fn test_case_async<'a, F, Fut>(f: F)
where
    F: Fn() -> Fut + 'a,
    Fut: Future<Output = ()> + 'a,
{
    with_tls(async move {
        let sections = Sections::new();
        while !sections.completed() {
            let section = sections.root();
            let _guard = Guard::set(Some(Box::new(section)));
            f().await;
        }
    })
    .await
}

pub(crate) fn with_tls<Fut: Future>(fut: Fut) -> impl Future<Output = Fut::Output> {
    WithTls { fut, cache: None }
}

#[pin_project]
#[must_use]
struct WithTls<Fut> {
    #[pin]
    fut: Fut,
    cache: Option<Box<Section>>,
}

impl<Fut> Future for WithTls<Fut>
where
    Fut: Future,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();

        let _guard = Guard::set(me.cache.take());

        let polled = me.fut.poll(cx);
        if let Poll::Pending = polled {
            *me.cache = crate::tls::take();
        }
        polled
    }
}
