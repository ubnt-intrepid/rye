#![cfg(feature = "futures")]

use crate::section::Section;
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::pin::Pin;

impl Section {
    pub async fn set_async<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        WithSectionAsync { fut, section: self }.await
    }
}

#[pin_project]
struct WithSectionAsync<'a, Fut> {
    #[pin]
    fut: Fut,
    section: &'a mut Section,
}

impl<Fut> Future for WithSectionAsync<'_, Fut>
where
    Fut: Future,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let fut = me.fut;
        let section = me.section;
        section.set(|| fut.poll(cx))
    }
}
