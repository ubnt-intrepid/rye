use crate::{
    location::Location,
    reporter::{Outcome, Reporter},
    termination::Termination,
    test::{SectionId, TestPlan},
};
use futures_core::future::Future;
use std::{fmt, marker::PhantomData, ptr::NonNull};

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

#[doc(hidden)] // private API.
#[repr(transparent)]
pub struct ContextPtr(NonNull<Context<'static>>);

unsafe impl Send for ContextPtr {}

impl ContextPtr {
    #[doc(hidden)] // private API.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_mut(&mut self) -> &mut Context<'static> {
        unsafe { self.0.as_mut() }
    }
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

    #[inline]
    unsafe fn transmute(&mut self) -> ContextPtr {
        ContextPtr(NonNull::from(&mut *self).cast::<Context<'static>>())
    }

    #[cfg(test)]
    pub(crate) fn current_section_name(&self) -> Option<&'static str> {
        self.current_section.map(|section| section.name)
    }

    pub(crate) async fn run_async<Fut>(&mut self, f: fn(ContextPtr) -> Fut) -> Result<(), Outcome>
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let outcome = f(unsafe { self.transmute() }).await;
        self.check_outcome(outcome)
    }

    pub(crate) fn run_blocking(
        &mut self,
        f: fn(ContextPtr) -> anyhow::Result<()>,
    ) -> Result<(), Outcome> {
        let outcome = f(unsafe { self.transmute() });
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

// ==== private macros ====

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __enter_section {
    ( $ctx:ident, $id:expr, $name:expr, $(#[$attr:meta])* $block:block ) => {
        $(#[$attr])*
        {
            const SECTION: $crate::_internal::Section = $crate::_internal::Section {
                id: $id,
                name: $name,
                location: $crate::_internal::location!(),
            };
            let section = $ctx.enter_section(&SECTION);
            if section.enabled() {
                $block
            }
            $ctx.leave_section(section);
        }
    };
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __skip {
    ($ctx:ident) => {
        $crate::__skip!($ctx, "explicitly skipped");
    };
    ($ctx:ident, $($arg:tt)+) => {
        const LOCATION: $crate::_internal::Location = $crate::_internal::location!();
        return $ctx.skip(&LOCATION, format_args!($($arg)+));
    };
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __fail {
    ($ctx:ident) => {
        $crate::__fail!($ctx:ident, "explicitly failed");
    };
    ($ctx:ident, $($arg:tt)+) => {{
        const LOCATION: $crate::_internal::Location = $crate::_internal::location!();
        return $ctx.fail(&LOCATION, format_args!($($arg)+));
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __require {
    ($ctx:ident, $e:expr) => {
        if !($e) {
            const LOCATION: $crate::_internal::Location = $crate::_internal::location!();
            return $ctx.assertion_failed(
                &LOCATION,
                format_args!(concat!("assertion failed: ", stringify!($e))),
            );
        }
    };
}
