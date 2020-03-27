use crate::{
    location::Location,
    report::{Outcome, Reporter},
    test::{SectionId, TestPlan},
};
use maybe_unwind::Unwind;
use std::{fmt, marker::PhantomData, ptr::NonNull};

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
    outcome: Option<Outcome>,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(reporter: &'a mut (dyn Reporter + Send), plan: &'a TestPlan) -> Self {
        Self {
            plan,
            reporter,
            current_section: None,
            outcome: None,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) unsafe fn transmute(&mut self) -> ContextPtr {
        ContextPtr(NonNull::from(&mut *self).cast::<Context<'static>>())
    }

    #[cfg(test)]
    pub(crate) fn current_section_name(&self) -> Option<&'static str> {
        self.current_section.map(|section| section.name)
    }

    pub(crate) fn check_outcome(
        &mut self,
        result: Result<anyhow::Result<()>, Unwind>,
    ) -> Option<Outcome> {
        match result {
            Ok(Ok(())) => {
                debug_assert!(self.outcome.is_none());
                None
            }
            Ok(Err(err)) => Some(Outcome::Errored(err)),
            Err(unwind) => match self.outcome.take() {
                outcome @ Some(..) => outcome,
                None => Some(Outcome::Panicked(unwind)),
            },
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

    #[inline]
    fn exit(&mut self) -> ! {
        panic!("test function is explicitly terminated")
    }

    #[doc(hidden)] // private API.
    #[inline(never)]
    pub fn skip(&mut self, location: &'static Location, reason: fmt::Arguments<'_>) -> ! {
        debug_assert!(self.outcome.is_none());
        self.outcome.replace(Outcome::Skipped {
            location,
            reason: reason.to_string(),
        });
        self.exit()
    }

    #[doc(hidden)] // private API.
    #[inline(never)]
    pub fn fail(&mut self, location: &'static Location, reason: fmt::Arguments<'_>) -> ! {
        debug_assert!(self.outcome.is_none());
        self.outcome.replace(Outcome::Failed {
            location,
            reason: reason.to_string(),
        });
        self.exit()
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

/// Mark the current test case as skipped and then terminate its execution.
///
/// This macro can usually be used to disable some test cases that may not
/// success depending on the runtime context, such as network access or a
/// certain secret variables is not set.
#[macro_export]
macro_rules! skip {
    ( $ctx:ident ) => {
        $crate::skip!($ctx, "explicitly skipped");
    };
    ( $ctx:ident, $($arg:tt)+ ) => {{
        use $crate::_test_reexports as __rye;
        const LOCATION: __rye::Location = __rye::location!();
        $ctx.skip(&LOCATION, __rye::format_args!($($arg)+))
    }};
}

/// Mark the current test case as failed and then terminate its execution.
#[macro_export]
macro_rules! fail {
    ($ctx:ident) => {
        $crate::fail!($ctx:ident, "explicitly failed");
    };
    ($ctx:ident, $($arg:tt)+) => {{
        use $crate::_test_reexports as __rye;
        const LOCATION: __rye::Location = __rye::location!();
        $ctx.fail(&LOCATION, __rye::format_args!($($arg)+))
    }};
}
