use crate::{section::Sections, tls::Guard};

#[derive(Debug)]
#[must_use]
pub struct TestCase {
    sections: Sections,
}

impl Default for TestCase {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TestCase {
    /// Create a test case.
    pub fn new() -> Self {
        Self {
            sections: Sections::new(),
        }
    }

    pub fn completed(&self) -> bool {
        self.sections.completed()
    }

    /// Run the test case.
    pub fn run<F>(&mut self, f: F)
    where
        F: FnOnce(),
    {
        if self.completed() {
            return;
        }

        let section = self.sections.root();
        let _guard = Guard::set(Some(Box::new(section)));
        f();
    }
}

#[cfg(feature = "futures")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "futures")))]
mod futures {
    use super::*;
    use crate::section::Section;
    use futures_core::{
        future::Future,
        task::{self, Poll},
    };
    use pin_project::pin_project;
    use std::pin::Pin;

    impl TestCase {
        /// Run the test case asynchronously.
        pub async fn run_async<Fut>(&mut self, fut: Fut)
        where
            Fut: Future<Output = ()>,
        {
            if self.completed() {
                return;
            }

            with_tls(async move {
                let section = self.sections.root();
                let _guard = Guard::set(Some(Box::new(section)));
                fut.await;
            })
            .await
        }
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
}