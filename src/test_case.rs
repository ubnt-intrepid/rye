use futures::future::BoxFuture;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct TestCase {
    pub desc: TestDesc,
    pub test_fn: TestFn,
}

#[derive(Debug)]
pub enum TestFn {
    SyncTest(fn()),
    AsyncTest(fn() -> BoxFuture<'static, ()>),
}

/// Description about a test case.
#[derive(Debug)]
pub struct TestDesc {
    pub name: &'static str,
    pub module_path: &'static str,
    pub ignored: bool,
    pub sections: HashMap<SectionId, Section>,
    pub leaf_sections: &'static [SectionId],
}

pub(crate) type SectionId = u64;

#[derive(Debug)]
pub struct Section {
    pub name: &'static str,
    pub ancestors: HashSet<SectionId>,
}
