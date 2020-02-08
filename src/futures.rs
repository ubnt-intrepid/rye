#![cfg(feature = "futures")]

use crate::{section::Section, test_case::TestCase};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::pin::Pin;

#[inline]
pub async fn run_async<F, Fut>(f: F)
where
    F: Fn() -> Fut,
    Fut: Future,
{
    let test_case = TestCase::new("root");
    while !test_case.completed() {
        let mut section = test_case.root_section();
        section.scope_async(f()).await;
    }
}

impl Section {
    #[doc(hidden)] // private API.
    pub async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
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
        section.scope(|| fut.poll(cx))
    }
}
