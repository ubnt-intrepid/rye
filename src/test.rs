use crate::location::Location;
use futures_core::future::{BoxFuture, LocalBoxFuture};

#[allow(missing_docs)]
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

#[doc(hidden)] // private API.
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

#[allow(missing_docs)]
#[derive(Debug)]
pub enum TestFn {
    Async(fn() -> BoxFuture<'static, anyhow::Result<()>>),
    AsyncLocal(fn() -> LocalBoxFuture<'static, anyhow::Result<()>>),
    Blocking(fn() -> anyhow::Result<()>),
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __test_fn {
    (@async $path:path) => {
        $crate::_internal::TestFn::Async(|| {
            use $crate::_internal::{Box, Termination};
            let fut = $path();
            Box::pin(async move { Termination::into_result(fut.await) })
        })
    };

    (@async_local $path:path) => {
        $crate::_internal::TestFn::AsyncLocal(|| {
            use $crate::_internal::{Box, Termination};
            let fut = $path();
            Box::pin(async move { Termination::into_result(fut.await) })
        })
    };

    (@blocking $path:path) => {
        $crate::_internal::TestFn::Blocking(|| {
            use $crate::_internal::Termination;
            Termination::into_result($path())
        })
    };
}

#[doc(hidden)] // private API
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TestName {
    pub raw: &'static str,
}

impl AsRef<str> for TestName {
    fn as_ref(&self) -> &str {
        self.raw.splitn(2, "::").nth(1).unwrap()
    }
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __test_name {
    ($name:ident) => {
        $crate::_internal::TestName {
            raw: $crate::_internal::concat!(
                $crate::_internal::module_path!(),
                "::",
                $crate::_internal::stringify!($name),
            ),
        }
    };
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __register_test_case {
    ($target:ident) => {
        $crate::_internal::paste::item! {
            $crate::_internal::cfg_harness! {
                #[$crate::_internal::linkme::distributed_slice($crate::_internal::TEST_CASES)]
                #[linkme(crate = $crate::_internal::linkme)]
                #[allow(non_upper_case_globals)]
                static [< __TEST_CASE_HARNESS__ $target >]: &dyn $crate::_internal::TestCase = $target;
            }
            $crate::_internal::cfg_frameworks! {
                #[test_case]
                const [< __TEST_CASE_FRAMEWORKS__ $target >]: &dyn $crate::_internal::TestCase = $target;
            }
        }
    };
}

#[doc(hidden)] // private API.
#[cfg(not(feature = "harness"))]
#[macro_export]
macro_rules! __cfg_harness {
    ($($item:item)*) => {};
}

#[doc(hidden)] // private API.
#[cfg(feature = "harness")]
#[macro_export]
macro_rules! __cfg_harness {
    ($($item:item)*) => ( $($item)* );
}

#[doc(hidden)] // private API.
#[cfg(not(feature = "frameworks"))]
#[macro_export]
macro_rules! __cfg_frameworks {
    ($($item:item)*) => {};
}

#[doc(hidden)] // private API.
#[cfg(feature = "frameworks")]
#[macro_export]
macro_rules! __cfg_frameworks {
    ($($item:item)*) => ( $($item)* );
}
