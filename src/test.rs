//! Registration of test cases.

use self::imp::{Section, SectionId, TestFn};
use std::{collections::HashMap, error, fmt};

#[doc(hidden)] // private API.
#[derive(Debug)]
pub struct Location {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
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
        $crate::test::Location {
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}

/// Description about a single test case.
#[derive(Debug)]
pub struct Test {
    pub(crate) desc: &'static TestDesc,
    pub(crate) test_fn: TestFn,
    pub(crate) filtered_out: bool,
}

impl Test {
    /// Return the reference to the test description.
    #[inline]
    pub fn desc(&self) -> &'static TestDesc {
        self.desc
    }

    /// Return the test case is asynchronous or not.
    #[inline]
    pub fn is_async(&self) -> bool {
        match self.test_fn {
            TestFn::Async { .. } => true,
            _ => false,
        }
    }

    /// Return whether the future produced by the test case must
    /// be executed onto the current thread or not.
    #[inline]
    pub fn is_local(&self) -> bool {
        match self.test_fn {
            TestFn::Async { local, .. } => local,
            TestFn::Blocking { .. } => false,
        }
    }
}

/// Metadata about a test case.
#[derive(Debug)]
pub struct TestDesc {
    #[doc(hidden)]
    pub module_path: &'static str,
    #[doc(hidden)]
    pub location: Location,
    #[doc(hidden)]
    pub todo: bool,
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
        self.module_path
            .splitn(2, "::")
            .nth(1)
            .unwrap_or("<unknown>")
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

impl<E> Fallible for Result<(), E> where E: Into<Box<dyn error::Error + Send + Sync + 'static>> {}

/// A collection of one or more test cases.
pub trait TestSet: Send + Sync {
    /// Register a collection of test cases in the registry.
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError>;
}

impl<T: ?Sized> TestSet for &T
where
    T: TestSet,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        (**self).register(registry)
    }
}

impl<T: ?Sized> TestSet for Box<T>
where
    T: TestSet,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        (**self).register(registry)
    }
}

impl<T> TestSet for [T]
where
    T: TestSet,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        for tests in self {
            tests.register(registry)?;
        }
        Ok(())
    }
}

/// The registry of test cases.
pub trait Registry {
    #[doc(hidden)] // private API.
    fn add_test(&mut self, desc: &'static TestDesc, test_fn: TestFn) -> Result<(), RegistryError>;
}

impl<R: ?Sized> Registry for &mut R
where
    R: Registry,
{
    #[doc(hidden)] // private API.
    #[inline]
    fn add_test(&mut self, desc: &'static TestDesc, test_fn: TestFn) -> Result<(), RegistryError> {
        (**self).add_test(desc, test_fn)
    }
}

impl<R: ?Sized> Registry for Box<R>
where
    R: Registry,
{
    #[doc(hidden)] // private API.
    #[inline]
    fn add_test(&mut self, desc: &'static TestDesc, test_fn: TestFn) -> Result<(), RegistryError> {
        (**self).add_test(desc, test_fn)
    }
}

/// The error value occurred during registration test cases.
#[derive(Debug, thiserror::Error)]
#[error("{}", _0)]
pub struct RegistryError(#[source] Box<dyn error::Error + Send + Sync>);

impl RegistryError {
    /// Create a new `RegistryError`.
    #[inline]
    pub fn new(cause: impl Into<Box<dyn error::Error + Send + Sync>>) -> Self {
        Self(cause.into())
    }
}

#[allow(missing_docs)]
pub(crate) mod imp {
    use super::Fallible;
    use futures::{
        future::{Future, LocalFutureObj},
        task::{self, FutureObj, Poll},
    };
    use pin_project::pin_project;
    use std::{collections::HashSet, error, pin::Pin};

    pub trait FallibleImp {
        fn is_ok(&self) -> bool;
        fn into_result(self: Box<Self>) -> Result<(), Box<dyn error::Error + Send + Sync>>;
    }

    impl FallibleImp for () {
        fn is_ok(&self) -> bool {
            true
        }

        fn into_result(self: Box<Self>) -> Result<(), Box<dyn error::Error + Send + Sync>> {
            Ok(())
        }
    }

    impl<E> FallibleImp for Result<(), E>
    where
        E: Into<Box<dyn error::Error + Send + Sync + 'static>>,
    {
        fn is_ok(&self) -> bool {
            self.is_ok()
        }

        fn into_result(self: Box<Self>) -> Result<(), Box<dyn error::Error + Send + Sync>> {
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
        Blocking { f: fn() -> Box<dyn Fallible> },
        Async { f: fn() -> TestFuture, local: bool },
    }

    #[pin_project]
    pub struct TestFuture {
        #[pin]
        inner: LocalFutureObj<'static, Box<dyn Fallible>>,
        local: bool,
    }

    impl TestFuture {
        #[inline]
        pub fn new<Fut>(fut: Fut) -> Self
        where
            Fut: Future + Send + 'static,
            Fut::Output: Fallible,
        {
            Self {
                inner: LocalFutureObj::new(Box::pin(async move {
                    Box::new(fut.await) as Box<dyn Fallible>
                })),
                local: false,
            }
        }

        #[inline]
        pub fn new_local<Fut>(fut: Fut) -> Self
        where
            Fut: Future + 'static,
            Fut::Output: Fallible,
        {
            Self {
                inner: LocalFutureObj::new(Box::pin(async move {
                    Box::new(fut.await) as Box<dyn Fallible>
                })),
                local: true,
            }
        }

        #[inline]
        pub(crate) fn into_future_obj(self) -> FutureObj<'static, Box<dyn Fallible>> {
            assert!(
                !self.local,
                "the test future cannot be converted into FutureObj when it is not Send"
            );
            unsafe { self.inner.into_future_obj() }
        }
    }

    impl Future for TestFuture {
        type Output = Box<dyn Fallible>;

        #[inline]
        fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
            let me = self.project();
            me.inner.poll(cx)
        }
    }
}
