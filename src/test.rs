use futures::{
    future::{Future, LocalFutureObj},
    task::{self, FutureObj, Poll},
};
use pin_project::pin_project;
use std::{
    collections::{HashMap, HashSet},
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
    #[inline]
    pub fn name(&self) -> &str {
        self.desc
            .module_path
            .splitn(2, "::")
            .nth(1)
            .unwrap_or("<unknown>")
    }

    pub fn is_async(&self) -> bool {
        match self.test_fn {
            TestFn::AsyncTest { .. } => true,
            _ => false,
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
    SyncTest(fn()),
    AsyncTest { f: fn() -> TestFuture, local: bool },
}

#[doc(hidden)] // private API.
#[pin_project]
pub struct TestFuture {
    #[pin]
    inner: LocalFutureObj<'static, ()>,
    local: bool,
}

impl TestFuture {
    #[doc(hidden)] // private API.
    #[inline]
    pub fn new<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            inner: LocalFutureObj::new(Box::pin(fut)),
            local: false,
        }
    }

    #[doc(hidden)] // private API.
    #[inline]
    pub fn new_local<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = ()> + 'static,
    {
        Self {
            inner: LocalFutureObj::from(Box::pin(fut)),
            local: true,
        }
    }

    #[inline]
    pub(crate) fn into_future_obj(self) -> FutureObj<'static, ()> {
        assert!(
            !self.local,
            "the test future cannot be converted into FutureObj when it is not Send"
        );
        unsafe { self.inner.into_future_obj() }
    }
}

impl Future for TestFuture {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        me.inner.poll(cx)
    }
}
