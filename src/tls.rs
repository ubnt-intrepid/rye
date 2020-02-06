use crate::section::Section;
use std::{cell::Cell, marker::PhantomData, pin::Pin};

thread_local! {
    static CURRENT_SECTION: Cell<Option<Box<Section>>> = Cell::new(None);
}

struct SetOnDrop(Option<Box<Section>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CURRENT_SECTION.with(|tls| {
            tls.set(self.0.take());
        });
    }
}

pub struct Guard<'a> {
    _set_on_drop: SetOnDrop,
    _marker: PhantomData<Pin<&'a mut Section>>,
}

pub fn set<'a>(section: Section) -> Guard<'a> {
    let old_section = CURRENT_SECTION.with(|tls| tls.replace(Some(Box::new(section))));
    Guard {
        _set_on_drop: SetOnDrop(old_section),
        _marker: PhantomData,
    }
}

pub(crate) fn with<F, R>(f: F) -> R
where
    F: FnOnce(&mut Section) -> R,
{
    let section_ptr = CURRENT_SECTION.with(|tls| tls.replace(None));
    let mut reset = SetOnDrop(section_ptr);
    let section_ptr = reset.0.as_deref_mut().expect("current section is not set");
    f(section_ptr)
}

#[cfg(feature = "futures")]
pub(crate) mod futures {
    use super::*;
    use futures_core::{
        future::Future,
        task::{self, Poll},
    };
    use pin_project::pin_project;
    use std::pin::Pin;

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

            let prev_section = CURRENT_SECTION.with(|tls| tls.replace(me.cache.take()));
            let _reset = SetOnDrop(prev_section);

            let polled = me.fut.poll(cx);
            if let Poll::Pending = polled {
                *me.cache = CURRENT_SECTION.with(|tls| tls.take());
            }
            polled
        }
    }
}
