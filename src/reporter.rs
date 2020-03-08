#![allow(missing_docs)]

mod console;
mod log;

pub use self::{console::ConsoleReporter, log::LogReporter};

use crate::test::{Test, TestDesc};

#[derive(Debug)]
pub(crate) enum TestResult {
    Passed,
    Failed,
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct TestCaseSummary {
    pub(crate) desc: &'static TestDesc,
    pub(crate) result: TestResult,
    pub(crate) error_message: Option<String>,
}

impl TestCaseSummary {
    pub(crate) fn is_passed(&self) -> bool {
        match self.result {
            TestResult::Passed => true,
            _ => false,
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
        self.failed.is_empty()
    }

    #[allow(missing_docs)]
    pub fn append(&mut self, result: TestCaseSummary) {
        match result.result {
            TestResult::Passed => {
                self.passed.push(result);
            }
            TestResult::Failed => {
                self.failed.push(result);
            }
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
