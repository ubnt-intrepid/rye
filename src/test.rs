use futures::future::BoxFuture;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

/// Data about a single test case.
#[derive(Debug)]
pub struct Test {
    pub desc: TestDesc,
    pub test_fn: TestFn,
}

/// Description about a test.
#[derive(Debug, Clone)]
pub struct TestDesc {
    /// The name of the test.
    pub name: Cow<'static, str>,

    /// A collection of sections described in the test.
    pub sections: HashMap<SectionId, Section>,

    /// A collection of section IDs to be run.
    pub leaf_sections: Vec<SectionId>,
}

pub(crate) type SectionId = u64;

#[derive(Debug, Clone)]
pub struct Section {
    pub name: &'static str,
    pub ancestors: HashSet<SectionId>,
}

/// The body of test function.
#[derive(Debug)]
pub enum TestFn {
    SyncTest(fn()),
    AsyncTest(fn() -> BoxFuture<'static, ()>),
}
