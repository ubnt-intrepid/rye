use crate::{
    location::Location,
    reporter::{Outcome, Reporter},
    termination::Termination,
    test::{SectionId, TestPlan},
};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::{cell::Cell, fmt, marker::PhantomData, mem, pin::Pin, ptr::NonNull};

#[derive(Debug)]
enum ExitReason {
    Skipped {
        location: &'static Location,
        reason: String,
    },
    Failed {
        location: &'static Location,
        reason: String,
    },
    AssertionFailed {
        location: &'static Location,
        message: String,
    },
}

/// Context values while running the test case.
pub struct Context<'a> {
    plan: &'a TestPlan,
    #[allow(dead_code)]
    reporter: &'a mut (dyn Reporter + Send),
    current_section: Option<&'static Section>,
    exit_reason: Option<ExitReason>,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(reporter: &'a mut (dyn Reporter + Send), plan: &'a TestPlan) -> Self {
        Self {
            plan,
            reporter,
            current_section: None,
            exit_reason: None,
            _marker: PhantomData,
        }
    }

    #[doc(hidden)] // private API
    pub fn enter_section(&mut self, section: &'static Section) -> EnterSection {
        let enabled = self.plan.is_enabled(section.id);
        let last_section = self.current_section.replace(section);
        EnterSection {
            enabled,
            last_section,
        }
    }

    #[doc(hidden)] // private API
    pub fn leave_section(&mut self, enter: EnterSection) {
        self.current_section = enter.last_section;
    }

    #[cfg(test)]
    pub(crate) fn current_section_name(&self) -> Option<&'static str> {
        self.current_section.map(|section| section.name)
    }

    pub(crate) async fn run_async<Fut>(&mut self, fut: Fut) -> Result<(), Outcome>
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let outcome = self.scope_async(fut).await;
        self.check_outcome(outcome)
    }

    pub(crate) fn run_blocking(&mut self, f: fn() -> anyhow::Result<()>) -> Result<(), Outcome> {
        let outcome = self.scope(f);
        self.check_outcome(outcome)
    }

    fn check_outcome(&mut self, result: anyhow::Result<()>) -> Result<(), Outcome> {
        match result {
            Ok(()) => match self.exit_reason.take() {
                Some(ExitReason::Skipped { location, reason }) => {
                    Err(Outcome::Skipped { location, reason })
                }
                Some(ExitReason::Failed { location, reason }) => {
                    Err(Outcome::Failed { location, reason })
                }
                Some(ExitReason::AssertionFailed { location, message }) => {
                    Err(Outcome::AssertionFailed { location, message })
                }
                None => Ok(()),
            },
            Err(err) => Err(Outcome::Errored(err)),
        }
    }

    #[doc(hidden)] // private API.
    #[inline(never)]
    pub fn skip<T>(&mut self, location: &'static Location, reason: fmt::Arguments<'_>) -> T
    where
        T: Termination,
    {
        debug_assert!(self.exit_reason.is_none());
        self.exit_reason.replace(ExitReason::Skipped {
            location,
            reason: reason.to_string(),
        });
        T::exit()
    }

    #[doc(hidden)] // private API.
    #[inline(never)]
    pub fn fail<T>(&mut self, location: &'static Location, reason: fmt::Arguments<'_>) -> T
    where
        T: Termination,
    {
        debug_assert!(self.exit_reason.is_none());
        self.exit_reason.replace(ExitReason::Failed {
            location,
            reason: reason.to_string(),
        });
        T::exit()
    }

    #[doc(hidden)] // private API.
    #[inline(never)]
    pub fn assertion_failed<T>(
        &mut self,
        location: &'static Location,
        message: fmt::Arguments<'_>,
    ) -> T
    where
        T: Termination,
    {
        debug_assert!(self.exit_reason.is_none());
        self.exit_reason.replace(ExitReason::AssertionFailed {
            location,
            message: message.to_string(),
        });
        T::exit()
    }
}

#[doc(hidden)]
pub struct EnterSection {
    enabled: bool,
    last_section: Option<&'static Section>,
}

impl EnterSection {
    #[doc(hidden)]
    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

#[doc(hidden)] // private API
pub struct Section {
    pub id: SectionId,
    pub name: &'static str,
    pub location: Location,
}

// ==== TLS ====

thread_local! {
    static TLS_CTX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None);
}

struct Guard(Option<NonNull<Context<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| tls.set(self.0.take()));
    }
}

#[doc(hidden)] // private API
#[inline]
pub fn with_tls_context<F, R>(f: F) -> R
where
    F: FnOnce(&mut Context<'_>) -> R,
{
    let ctx_ptr = TLS_CTX.with(|tls| tls.take());
    let _guard = Guard(ctx_ptr);
    let mut ctx_ptr = ctx_ptr.expect("cannot acquire the test context");
    unsafe { f(ctx_ptr.as_mut()) }
}

impl Context<'_> {
    fn scope<F, R>(&mut self, f: F) -> R
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
    async fn scope_async<Fut>(&mut self, fut: Fut) -> Fut::Output
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
}

// ==== private macros ====

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __enter_section {
    ( $id:expr, $name:expr, $(#[$attr:meta])* $block:block ) => {
        $(#[$attr])*
        {
            const SECTION: $crate::_internal::Section = $crate::_internal::Section {
                id: $id,
                name: $name,
                location: $crate::_internal::location!(),
            };
            let section = $crate::_internal::with_tls_context(|ctx| ctx.enter_section(&SECTION));
            if section.enabled() {
                $block
            }
            $crate::_internal::with_tls_context(|ctx| ctx.leave_section(section));
        }
    };
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __skip {
    () => ( $crate::__skip!("explicitly skipped") );
    ($($arg:tt)+) => {
        const LOCATION: $crate::_internal::Location = $crate::_internal::location!();
        return $crate::_internal::with_tls_context(|ctx| {
            ctx.skip(&LOCATION, format_args!($($arg)+))
        });
    };
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __fail {
    () => ( $crate::__fail!("explicitly failed") );
    ($($arg:tt)+) => {{
        const LOCATION: $crate::_internal::Location = $crate::_internal::location!();
        return $crate::_internal::with_tls_context(|ctx| {
            ctx.fail(&LOCATION, format_args!($($arg)+))
        });
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __require {
    ($e:expr) => {
        if !($e) {
            const LOCATION: $crate::_internal::Location = $crate::_internal::location!();
            return $crate::_internal::with_tls_context(|ctx| {
                ctx.assertion_failed(
                    &LOCATION,
                    format_args!(concat!("assertion failed: ", stringify!($e))),
                )
            });
        }
    };
}
