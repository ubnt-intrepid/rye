use crate::test::imp::{SectionId, TestDesc};
use futures::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::{cell::Cell, marker::PhantomData, mem, pin::Pin, ptr::NonNull};

/// Context values while running the test case.
pub struct Context<'a> {
    pub(crate) desc: &'a TestDesc,
    pub(crate) target_section: Option<SectionId>,
    pub(crate) current_section: Option<SectionId>,
    pub(crate) _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

thread_local! {
    static TLS_CTX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None);
}

struct Guard(Option<NonNull<Context<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| tls.set(self.0.take()));
    }
}

impl<'a> Context<'a> {
    pub(crate) fn scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let prev = TLS_CTX.with(|tls| unsafe {
            let ctx_ptr = mem::transmute::<&mut Self, &mut Context<'static>>(self);
            tls.replace(Some(NonNull::from(ctx_ptr)))
        });
        let _guard = Guard(prev);
        f()
    }

    #[inline]
    pub(crate) async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        #[pin_project]
        struct ScopeAsync<'a, 'ctx, Fut> {
            #[pin]
            fut: Fut,
            ctx: &'a mut Context<'ctx>,
        }

        impl<Fut> Future for ScopeAsync<'_, '_, Fut>
        where
            Fut: Future,
        {
            type Output = Fut::Output;

            fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
                let me = self.project();
                let fut = me.fut;
                me.ctx.scope(|| fut.poll(cx))
            }
        }

        ScopeAsync { fut, ctx: self }.await
    }

    /// Return whether the test context is available or not.
    #[inline]
    pub fn is_set() -> bool {
        TLS_CTX.with(|tls| tls.get().is_some())
    }

    /// Attempt to get a reference to the test context and invoke the provided closure.
    ///
    /// This function returns an `AccessError` if the test context is not available.
    pub fn try_with<F, R>(f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&mut Context<'_>) -> R,
    {
        let ctx_ptr = TLS_CTX.with(|tls| tls.take());
        let _guard = Guard(ctx_ptr);
        let mut ctx_ptr = ctx_ptr.ok_or_else(|| AccessError { _p: () })?;
        Ok(unsafe { f(ctx_ptr.as_mut()) })
    }

    /// Get a reference to the test context and invoke the provided closure.
    ///
    /// # Panics
    /// This function causes a panic if the test context is not available.
    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Context<'_>) -> R,
    {
        Self::try_with(f).expect("cannot acquire the test context")
    }

    /// Return the name of section currently executing.
    #[inline]
    pub fn section_name(&self) -> Option<&str> {
        self.current_section.map(|id| self.desc.sections[&id].name)
    }

    pub(crate) fn enter_section(&mut self, id: SectionId) -> EnterSection {
        let enabled = self.target_section.map_or(false, |section_id| {
            let section = self
                .desc
                .sections
                .get(&section_id)
                .expect("invalid section id is set");
            section_id == id || section.ancestors.contains(&id)
        });
        let last_section = self.current_section.replace(id);
        EnterSection {
            enabled,
            last_section,
        }
    }
}

pub struct EnterSection {
    enabled: bool,
    last_section: Option<SectionId>,
}

impl EnterSection {
    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn leave(self) {
        Context::with(|ctx| {
            ctx.current_section = self.last_section;
        })
    }
}

/// The error value returned from `TestContext::try_with`.
#[derive(Debug, thiserror::Error)]
#[error("cannot access the test context outside of the test body")]
pub struct AccessError {
    _p: (),
}
