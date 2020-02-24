use futures::{
    future::{Future, LocalFutureObj},
    task::{self, FutureObj, Poll},
};
use pin_project::pin_project;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    pin::Pin,
};

/// Data about a single test case.
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

pub trait TestResult: 'static {
    fn is_success(&self) -> bool;

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
