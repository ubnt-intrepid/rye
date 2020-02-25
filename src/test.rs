//! Registration of test cases.

use futures::{
    future::{Future, LocalFutureObj},
    task::{self, FutureObj, Poll},
};
use pin_project::pin_project;
use std::{
    collections::{HashMap, HashSet},
    pin::Pin,
};
use std::{error, fmt};

/// Description about a single test case.
#[derive(Debug)]
pub struct Test {
    #[doc(hidden)] // private API.
    pub desc: TestDesc,
    #[doc(hidden)] // private API.
    pub test_fn: TestFn,
}

impl Test {
    /// Return the name of test case.
    ///
    /// Test cases are uniquely named by their relative path from
    /// the root module.
    #[inline]
    pub fn name(&self) -> &str {
        self.desc
            .module_path
            .splitn(2, "::")
            .nth(1)
            .unwrap_or("<unknown>")
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

#[doc(hidden)] // private API.
#[derive(Debug, Clone)]
pub struct TestDesc {
    pub module_path: &'static str,
    pub sections: HashMap<SectionId, Section>,
    pub leaf_sections: Vec<SectionId>,
}

pub(crate) type SectionId = u64;

#[doc(hidden)] // private API.
#[derive(Debug, Clone)]
pub struct Section {
    pub name: &'static str,
    pub ancestors: HashSet<SectionId>,
}

#[doc(hidden)] // private API.
#[derive(Debug)]
pub enum TestFn {
    Blocking { f: fn() -> Box<dyn TestResult> },
    Async { f: fn() -> TestFuture, local: bool },
}

#[doc(hidden)] // private API.
#[pin_project]
pub struct TestFuture {
    #[pin]
    inner: LocalFutureObj<'static, Box<dyn TestResult>>,
    local: bool,
}

impl TestFuture {
    #[doc(hidden)] // private API.
    #[inline]
    pub fn new<Fut>(fut: Fut) -> Self
    where
        Fut: Future + Send + 'static,
        Fut::Output: TestResult,
    {
        Self {
            inner: LocalFutureObj::new(Box::pin(async move {
                Box::new(fut.await) as Box<dyn TestResult>
            })),
            local: false,
        }
    }

    #[doc(hidden)] // private API.
    #[inline]
    pub fn new_local<Fut>(fut: Fut) -> Self
    where
        Fut: Future + 'static,
        Fut::Output: TestResult,
    {
        Self {
            inner: LocalFutureObj::new(Box::pin(async move {
                Box::new(fut.await) as Box<dyn TestResult>
            })),
            local: true,
        }
    }

    #[inline]
    pub(crate) fn into_future_obj(self) -> FutureObj<'static, Box<dyn TestResult>> {
        assert!(
            !self.local,
            "the test future cannot be converted into FutureObj when it is not Send"
        );
        unsafe { self.inner.into_future_obj() }
    }
}

impl Future for TestFuture {
    type Output = Box<dyn TestResult>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        me.inner.poll(cx)
    }
}

/// The result values returned from test functions.
pub trait TestResult: test_result::Sealed + 'static {
    /// Return `true` if the test function was successfully completed.
    fn is_success(&self) -> bool;

    /// Return a reference to the object for writing the error message.
    fn error_message(&self) -> Option<&(dyn fmt::Debug + 'static)> {
        None
    }
}

impl TestResult for () {
    fn is_success(&self) -> bool {
        true
    }
}

impl<E> TestResult for Result<(), E>
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

mod test_result {
    use super::*;

    pub trait Sealed {}

    impl Sealed for () {}

    impl<E> Sealed for Result<(), E> where E: fmt::Debug + 'static {}
}

/// The registration of one or more test cases.
pub trait Registration {
    /// Register a collection of test cases in the registry.
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError>;
}

impl<R: ?Sized> Registration for &R
where
    R: Registration,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        (**self).register(registry)
    }
}

impl<R: ?Sized> Registration for Box<R>
where
    R: Registration,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        (**self).register(registry)
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

#[derive(Debug)]
pub struct RegistryError(Box<dyn error::Error + Send + Sync>);

impl RegistryError {
    pub fn new(cause: impl Into<Box<dyn error::Error + Send + Sync>>) -> Self {
        Self(cause.into())
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}

impl error::Error for RegistryError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.0)
    }
}
