#![allow(missing_docs)]

mod console;
mod log;

pub use self::{console::ConsoleReporter, log::LogReporter};

use crate::test::{Fallible, Test, TestDesc};
use maybe_unwind::Unwind;
use std::error;

#[derive(Debug)]
enum ResultDisposition {
    Passed,
    Failed,
}

#[derive(Debug)]
struct Fail(FailKind);

#[derive(Debug)]
enum FailKind {
    Unwind(Unwind),
    Error(Box<dyn error::Error + Send + Sync + 'static>),
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct TestCaseSummary {
    desc: &'static TestDesc,
    result: ResultDisposition,
    fails: Vec<Fail>,
}

impl TestCaseSummary {
    pub(crate) fn new(desc: &'static TestDesc) -> Self {
        Self {
            desc,
            result: ResultDisposition::Passed,
            fails: vec![],
        }
    }

    pub(crate) fn is_passed(&self) -> bool {
        match self.result {
            ResultDisposition::Passed => true,
            ResultDisposition::Failed => false,
        }
    }

    pub(crate) fn check_result(&mut self, result: Result<Box<dyn Fallible>, Unwind>) {
        match result {
            Ok(result) => {
                if let Err(err) = result.into_result() {
                    self.result = ResultDisposition::Failed;
                    self.fails.push(Fail(FailKind::Error(err)));
                }
            }
            Err(unwind) => {
                self.result = ResultDisposition::Failed;
                self.fails.push(Fail(FailKind::Unwind(unwind)));
            }
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct Summary {
    pub(crate) passed: Vec<TestCaseSummary>,
    pub(crate) failed: Vec<TestCaseSummary>,
    pub(crate) filtered_out: Vec<&'static TestDesc>,
}

impl Summary {
    #[allow(missing_docs)]
    pub fn is_passed(&self) -> bool {
        self.failed.is_empty() || self.failed.iter().all(|summary| summary.desc.todo)
    }

    #[allow(missing_docs)]
    pub fn append(&mut self, result: TestCaseSummary) {
        if result.is_passed() {
            self.passed.push(result);
        } else {
            self.failed.push(result);
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
