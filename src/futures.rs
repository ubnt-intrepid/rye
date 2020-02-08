#![cfg(feature = "futures")]

use crate::section::Section;
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::pin::Pin;

pub async fn with_section_async<Fut>(section: &mut Section, fut: Fut) -> Fut::Output
where
    Fut: Future,
{
    WithSectionAsync { fut, section }.await
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
        crate::tls::set(section, || fut.poll(cx))
    }
}
