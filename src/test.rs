use crate::location::Location;
use futures_core::future::{BoxFuture, LocalBoxFuture};
use hashbrown::{HashMap, HashSet};
use std::borrow::Cow;

#[allow(missing_docs)]
pub trait TestCase: Send + Sync {
    fn desc(&self) -> TestDesc;
    #[doc(hidden)] // private API.
    fn test_fn(&self) -> TestFn;
}

impl<T: ?Sized> TestCase for &T
where
    T: TestCase,
{
    fn desc(&self) -> TestDesc {
        (**self).desc()
    }

    fn test_fn(&self) -> TestFn {
        (**self).test_fn()
    }
}

/// Metadata about a test case.
#[derive(Debug)]
pub struct TestDesc {
    #[doc(hidden)]
    pub name: Cow<'static, str>,
    #[doc(hidden)]
    pub location: Location,
    #[doc(hidden)]
    pub sections: HashMap<SectionId, Section>,
    #[doc(hidden)]
    pub leaf_sections: &'static [SectionId],
}

impl TestDesc {
    /// Return the name of test case.
    ///
    /// Test cases are uniquely named by their relative path from
    /// the root module.
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }

    /// Return the iterator over the section ids to be enabled.
    pub(crate) fn target_sections(&self) -> impl Iterator<Item = Option<SectionId>> + '_ {
        enum TargetSections<'a> {
            Root { terminated: bool },
            Leaves(std::slice::Iter<'a, SectionId>),
        }
        let mut target_sections = if self.leaf_sections.is_empty() {
            TargetSections::Root { terminated: false }
        } else {
            TargetSections::Leaves(self.leaf_sections.iter())
        };
        std::iter::from_fn(move || match target_sections {
            TargetSections::Root { ref mut terminated } => {
                if !*terminated {
                    *terminated = true;
                    Some(None)
                } else {
                    None
                }
            }
            TargetSections::Leaves(ref mut iter) => iter.next().map(|&section| Some(section)),
        })
    }
}

pub(crate) type SectionId = u64;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Section {
    pub name: &'static str,
    pub ancestors: HashSet<SectionId>,
}

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

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __test_name {
    ($name:ident) => {
        $crate::_internal::test_name(
            $crate::_internal::module_path!(),
            $crate::_internal::stringify!($name),
        )
    };
}

#[allow(missing_docs)]
#[inline(never)]
pub fn test_name(module_path: &'static str, name: &'static str) -> Cow<'static, str> {
    module_path
        .splitn(2, "::")
        .nth(1)
        .map_or(name.into(), |m| format!("{}::{}", m, name).into())
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __sections {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => {
        <[()]>::len(&[$($crate::__sections!(@single $rest)),*])
    };

    ($( $key:expr => ($name:expr, { $($ancestors:tt)* }); )*) => {
        {
            let _cap = $crate::__sections!(@count $($key),*);
            #[allow(clippy::let_and_return)]
            let mut _map = $crate::_internal::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $crate::_internal::Section {
                    name: $name,
                    ancestors: {
                        let _cap = $crate::__sections!(@count $($ancestors),*);
                        #[allow(clippy::let_and_return)]
                        let mut _set = $crate::_internal::HashSet::with_capacity(_cap);
                        $(
                            _set.insert($ancestors);
                        )*
                        _set
                    },
                });
            )*
            _map
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
