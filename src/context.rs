use crate::{
    location::Location,
    reporter::{Outcome, Reporter},
    termination::Termination,
    test::{SectionId, TestPlan},
};
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

    pub(crate) fn check_outcome(&mut self, result: anyhow::Result<()>) -> Option<Outcome> {
        match result {
            Ok(()) => self.outcome.take(),
            Err(err) => Some(Outcome::Errored(err)),
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
        debug_assert!(self.outcome.is_none());
        self.outcome.replace(Outcome::Skipped {
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
        debug_assert!(self.outcome.is_none());
        self.outcome.replace(Outcome::Failed {
            location,
            reason: reason.to_string(),
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
            $crate::__fail!($ctx, concat!("assertion failed: ", stringify!($e)));
        }
    };
}
