use crate::section::Section;
use std::{cell::Cell, marker::PhantomData, ptr::NonNull};

thread_local! {
    static CURRENT_SECTION: Cell<Option<NonNull<Section>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Section>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CURRENT_SECTION.with(|tls| {
            tls.set(self.0.take());
        });
    }
}

pub struct Guard<'a> {
    _set_on_drop: SetOnDrop,
    _marker: PhantomData<&'a mut Section>,
}

pub fn set(section: &mut Section) -> Guard<'_> {
    let old_section = CURRENT_SECTION.with(|tls| tls.replace(Some(NonNull::from(section))));
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
    let _reset = SetOnDrop(section_ptr);
    let mut section_ptr = section_ptr.expect("current section is not set");
    unsafe { f(section_ptr.as_mut()) }
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
        #[pin_project]
        #[must_use]
        pub struct WithTls<Fut> {
            #[pin]
            fut: Fut,
            cache: Option<NonNull<Section>>,
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
                    *me.cache = CURRENT_SECTION.with(|tls| tls.replace(None));
                }
                polled
            }
        }

        WithTls { fut, cache: None }
    }
}
