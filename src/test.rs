#![allow(missing_docs)]

use crate::{context::ContextPtr, location::Location};
use futures_core::future::{BoxFuture, LocalBoxFuture};
use linkme::distributed_slice;

#[distributed_slice]
pub static TEST_CASES: [&'static dyn TestCase] = [..];

pub trait TestCase: Send + Sync {
    fn desc(&self) -> &'static TestDesc;

    #[doc(hidden)] // private API.
    fn test_fn(&self) -> TestFn;

    #[doc(hidden)] // private API.
    fn test_plans(&self) -> &'static [TestPlan];
}

impl<T: ?Sized> TestCase for &T
where
    T: TestCase,
{
    fn desc(&self) -> &'static TestDesc {
        (**self).desc()
    }

    #[doc(hidden)] // private API.
    fn test_fn(&self) -> TestFn {
        (**self).test_fn()
    }

    #[doc(hidden)] // private API.
    fn test_plans(&self) -> &'static [TestPlan] {
        (**self).test_plans()
    }
}

/// Metadata about a test case.
#[derive(Debug)]
pub struct TestDesc {
    #[doc(hidden)]
    pub name: TestName,
    #[doc(hidden)]
    pub location: Location,
}

impl TestDesc {
    /// Return the name of test case.
    ///
    /// Test cases are uniquely named by their relative path from
    /// the root module.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

#[derive(Debug)]
pub struct TestPlan {
    pub target: Option<SectionId>,
    pub ancestors: &'static [SectionId],
}

impl TestPlan {
    pub(crate) fn is_enabled(&self, id: SectionId) -> bool {
        self.target.map_or(false, |target| target == id) || self.ancestors.contains(&id)
    }
}

pub(crate) type SectionId = u64;

#[derive(Debug)]
pub enum TestFn {
    Async(fn(ContextPtr) -> BoxFuture<'static, anyhow::Result<()>>),
    AsyncLocal(fn(ContextPtr) -> LocalBoxFuture<'static, anyhow::Result<()>>),
    Blocking(fn(ContextPtr) -> anyhow::Result<()>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TestName {
    pub raw: &'static str,
}

impl AsRef<str> for TestName {
    fn as_ref(&self) -> &str {
        self.raw.splitn(2, "::").nth(1).unwrap()
    }
}

// ==== private macros ====

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __test_name {
    ($name:ident) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestName {
            raw: __rye::concat!(__rye::module_path!(), "::", __rye::stringify!($name)),
        }
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __test_fn {
    (@async $path:path) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestFn::Async(|mut ctx_ptr| {
            __rye::Box::pin(async move {
                __rye::Termination::into_result($path(ctx_ptr.as_mut()).await)
            })
        })
    }};

    (@async_local $path:path) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestFn::AsyncLocal(|mut ctx_ptr| {
            __rye::Box::pin(async move {
                __rye::Termination::into_result($path(ctx_ptr.as_mut()).await)
            })
        })
    }};

    (@blocking $path:path) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestFn::Blocking(|mut ctx_ptr| {
            __rye::Termination::into_result($path(ctx_ptr.as_mut()))
        })
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __register_test_case {
    ($target:ident) => {
        $crate::_test_reexports::paste_item! {
            #[$crate::_test_reexports::distributed_slice($crate::_test_reexports::TEST_CASES)]
            #[linkme(crate = $crate::_test_reexports::linkme)]
            #[allow(non_upper_case_globals)]
            static [< __TEST_CASE_HARNESS__ $target >]: &dyn $crate::_test_reexports::TestCase = $target;
        }
    };
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __location {
    () => {{
        use $crate::_test_reexports as __rye;
        __rye::Location {
            file: __rye::file!(),
            line: __rye::line!(),
            column: __rye::column!(),
        }
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __section {
    ( $ctx:ident, $id:expr, $name:expr, $(#[$attr:meta])* $block:block ) => {
        $(#[$attr])*
        {
            use $crate::_test_reexports as __rye;

            const SECTION: __rye::Section = __rye::Section {
                id: $id,
                name: $name,
                location: __rye::location!(),
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
    ( $ctx:ident ) => {
        $crate::__skip!($ctx, "explicitly skipped");
    };
    ( $ctx:ident, $($arg:tt)+ ) => {{
        use $crate::_test_reexports as __rye;
        const LOCATION: __rye::Location = __rye::location!();
        return $ctx.skip(&LOCATION, __rye::format_args!($($arg)+));
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __fail {
    ($ctx:ident) => {
        $crate::__fail!($ctx:ident, "explicitly failed");
    };
    ($ctx:ident, $($arg:tt)+) => {{
        use $crate::_test_reexports as __rye;
        const LOCATION: __rye::Location = __rye::location!();
        return $ctx.fail(&LOCATION, __rye::format_args!($($arg)+));
    }};
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __require {
    ($ctx:ident, $e:expr) => {{
        use $crate::_test_reexports as __rye;
        if !($e) {
            $crate::__fail!(
                $ctx,
                __rye::concat!("assertion failed: ", __rye::stringify!($e))
            );
        }
    }};
}
