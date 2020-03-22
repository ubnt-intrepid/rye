//! Registration of test cases.

use self::imp::{Location, Section, SectionId, TestFn};
use hashbrown::HashMap;
use std::borrow::Cow;

#[allow(missing_docs)]
pub trait TestCase: Send + Sync {
    fn desc(&self) -> TestDesc;
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

/// The result values returned from test functions.
pub trait Fallible: imp::FallibleImp {}

impl Fallible for () {}

impl<E> Fallible for Result<(), E> where E: Into<anyhow::Error> {}

#[allow(missing_docs)]
pub(crate) mod imp {
    use super::Fallible;
    use futures::task::{FutureObj, LocalFutureObj};
    use hashbrown::HashSet;
    use std::{borrow::Cow, fmt, panic};

    pub trait FallibleImp {
        fn is_ok(&self) -> bool;
        fn into_result(self: Box<Self>) -> anyhow::Result<()>;
    }

    impl FallibleImp for () {
        fn is_ok(&self) -> bool {
            true
        }

        fn into_result(self: Box<Self>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    impl<E> FallibleImp for Result<(), E>
    where
        E: Into<anyhow::Error>,
    {
        fn is_ok(&self) -> bool {
            self.is_ok()
        }

        fn into_result(self: Box<Self>) -> anyhow::Result<()> {
            (*self).map_err(|e| e.into())
        }
    }

    pub(crate) type SectionId = u64;

    #[derive(Debug)]
    pub struct Section {
        pub name: &'static str,
        pub ancestors: HashSet<SectionId>,
    }

    #[derive(Debug)]
    pub enum TestFn {
        Blocking(fn() -> Box<dyn Fallible>),
        Async(fn() -> FutureObj<'static, Box<dyn Fallible>>),
        LocalAsync(fn() -> LocalFutureObj<'static, Box<dyn Fallible>>),
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __async_local_test_fn {
        ($path:path) => {
            $crate::_internal::TestFn::LocalAsync(|| {
                use $crate::_internal::{Box, Fallible, LocalFutureObj};
                let fut = $path();
                LocalFutureObj::new(Box::pin(async move {
                    let fallible = fut.await;
                    Box::new(fallible) as Box<dyn Fallible>
                }))
            })
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __async_test_fn {
        ($path:path) => {
            $crate::_internal::TestFn::Async(|| {
                use $crate::_internal::{Box, Fallible, FutureObj};
                let fut = $path();
                FutureObj::new(Box::pin(async move {
                    let fallible = fut.await;
                    Box::new(fallible) as Box<dyn Fallible>
                }))
            })
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __blocking_test_fn {
        ($path:path) => {
            $crate::_internal::TestFn::Blocking(|| {
                use $crate::_internal::{Box, Fallible};
                let fallible = $path();
                Box::new(fallible) as Box<dyn Fallible>
            })
        };
    }

    #[derive(Debug)]
    pub struct Location {
        pub file: Cow<'static, str>,
        pub line: u32,
        pub column: u32,
    }

    impl Location {
        #[inline]
        pub(crate) fn from_std(loc: &panic::Location<'_>) -> Self {
            Self {
                file: loc.file().to_string().into(),
                line: loc.line(),
                column: loc.column(),
            }
        }
    }

    impl fmt::Display for Location {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}:{}:{}", self.file, self.line, self.column)
        }
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __location {
        () => {
            $crate::_internal::Location {
                file: file!().into(),
                line: line!(),
                column: column!(),
            }
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

    #[inline]
    pub fn test_name(module_path: &'static str, name: &'static str) -> Cow<'static, str> {
        module_path
            .splitn(2, "::")
            .nth(1)
            .map_or(name.into(), |m| format!("{}::{}", m, name).into())
    }
}
