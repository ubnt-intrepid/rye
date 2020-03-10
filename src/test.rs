//! Registration of test cases.

use self::imp::{Section, SectionId, TestFn};
use std::{collections::HashMap, error, fmt};

/// Description about a single test case.
#[derive(Debug)]
pub struct Test {
    #[doc(hidden)] // private API.
    pub desc: &'static TestDesc,
    #[doc(hidden)] // private API.
    pub test_fn: TestFn,
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
    pub sections: HashMap<SectionId, Section>,
    #[doc(hidden)]
    pub leaf_sections: Vec<SectionId>,
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
}

/// The result values returned from test functions.
pub trait Fallible: imp::FallibleImp + 'static {}

impl Fallible for () {}

impl<E> Fallible for Result<(), E> where E: fmt::Debug + 'static {}

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
    /// Register a test case.
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError>;
}

impl<R: ?Sized> Registry for &mut R
where
    R: Registry,
{
    #[inline]
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        (**self).add_test(test)
    }
}

impl<R: ?Sized> Registry for Box<R>
where
    R: Registry,
{
    #[inline]
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        (**self).add_test(test)
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
    use std::{collections::HashSet, fmt, pin::Pin};

    pub trait FallibleImp {
        fn is_success(&self) -> bool;

        fn error_message(&self) -> Option<&(dyn fmt::Debug + 'static)>;
    }

    impl FallibleImp for () {
        fn is_success(&self) -> bool {
            true
        }

        fn error_message(&self) -> Option<&(dyn fmt::Debug + 'static)> {
            None
        }
    }

    impl<E> FallibleImp for Result<(), E>
    where
        E: fmt::Debug + 'static,
    {
        fn is_success(&self) -> bool {
            self.is_ok()
        }

        fn error_message(&self) -> Option<&(dyn fmt::Debug + 'static)> {
            self.as_ref()
                .err()
                .map(|e| e as &(dyn fmt::Debug + 'static))
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
