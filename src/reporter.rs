#![allow(missing_docs)]

mod console;
mod log;

pub use self::{console::ConsoleReporter, log::LogReporter};

use crate::test::{Location, Test, TestDesc};
use std::{any::Any, error, sync::Arc};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Status {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug)]
enum Failure {
    Unwind {
        payload: Box<dyn Any + Send + 'static>,
        location: Location,
    },
    Error(Box<dyn error::Error + Send + Sync + 'static>),
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct TestCaseSummary {
    desc: Arc<TestDesc>,
    status: Status,
    skip_reason: Option<String>,
    failures: Vec<Failure>,
}

impl TestCaseSummary {
    pub(crate) fn new(desc: Arc<TestDesc>) -> Self {
        Self {
            desc,
            status: Status::Passed,
            skip_reason: None,
            failures: vec![],
        }
    }

    pub(crate) fn status(&self) -> Status {
        self.status
    }

    pub(crate) fn should_terminate(&self) -> bool {
        match self.status() {
            Status::Passed => false,
            Status::Failed | Status::Skipped => true,
        }
    }

    pub(crate) fn mark_skipped(&mut self, reason: String) {
        self.status = Status::Skipped;
        self.skip_reason.replace(reason);
    }

    pub(crate) fn mark_errored(&mut self, err: Box<dyn error::Error + Send + Sync + 'static>) {
        self.status = Status::Failed;
        self.failures.push(Failure::Error(err));
    }

    pub(crate) fn mark_panicked(
        &mut self,
        payload: Box<dyn Any + Send + 'static>,
        location: Location,
    ) {
        self.status = Status::Failed;
        self.failures.push(Failure::Unwind { payload, location });
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Summary {
    pub(crate) passed: Vec<TestCaseSummary>,
    pub(crate) failed: Vec<TestCaseSummary>,
    pub(crate) skipped: Vec<TestCaseSummary>,
    pub(crate) filtered_out: Vec<Arc<TestDesc>>,
}

impl Summary {
    #[inline]
    pub(crate) fn empty() -> Self {
        Self {
            passed: vec![],
            failed: vec![],
            skipped: vec![],
            filtered_out: vec![],
        }
    }

    pub(crate) fn is_passed(&self) -> bool {
        self.failed.is_empty()
    }

    pub(crate) fn append(&mut self, result: TestCaseSummary) {
        match result.status() {
            Status::Passed => self.passed.push(result),
            Status::Failed => self.failed.push(result),
            Status::Skipped => self.skipped.push(result),
        }
    }
}

pub trait Reporter {
    fn test_run_starting(&self, tests: &[Test]);
    fn test_run_ended(&self, summary: &Summary);

    fn test_case_starting(&self, desc: &TestDesc);
    fn test_case_ended(&self, summary: &TestCaseSummary);
}

macro_rules! impl_reporter_body {
    () => {
        fn test_run_starting(&self, tests: &[Test]) {
            (**self).test_run_starting(tests)
        }

        fn test_run_ended(&self, summary: &Summary) {
            (**self).test_run_ended(summary)
        }

        fn test_case_starting(&self, desc: &TestDesc) {
            (**self).test_case_starting(desc)
        }

        fn test_case_ended(&self, summary: &TestCaseSummary) {
            (**self).test_case_ended(summary)
        }
    };
}

impl<R: ?Sized> Reporter for &R
where
    R: Reporter,
{
    impl_reporter_body!();
}

impl<R: ?Sized> Reporter for Box<R>
where
    R: Reporter,
{
    impl_reporter_body!();
}

impl<R: ?Sized> Reporter for std::rc::Rc<R>
where
    R: Reporter,
{
    impl_reporter_body!();
}

impl<R: ?Sized> Reporter for std::sync::Arc<R>
where
    R: Reporter,
{
    impl_reporter_body!();
}
