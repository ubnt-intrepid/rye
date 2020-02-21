use futures::future::BoxFuture;
use std::collections::{HashMap, HashSet};

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
            TestFn::AsyncTest(..) => true,
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
    AsyncTest(fn() -> BoxFuture<'static, ()>),
}
