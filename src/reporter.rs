#![allow(missing_docs)]

pub mod console;

use crate::{
    executor::result::{Summary, TestCaseResult},
    test::Test,
};

pub trait Reporter {
    type TestCaseReporter: TestCaseReporter;

    fn test_run_starting(&mut self, tests: &[Test]);
    fn test_run_ended(&mut self, summary: &Summary);

    fn test_case_reporter(&mut self) -> Self::TestCaseReporter;
}

impl<R: ?Sized> Reporter for &mut R
where
    R: Reporter,
{
    type TestCaseReporter = R::TestCaseReporter;

    fn test_run_starting(&mut self, tests: &[Test]) {
        (**self).test_run_starting(tests)
    }

    fn test_run_ended(&mut self, summary: &Summary) {
        (**self).test_run_ended(summary)
    }

    fn test_case_reporter(&mut self) -> Self::TestCaseReporter {
        (**self).test_case_reporter()
    }
}

impl<R: ?Sized> Reporter for Box<R>
where
    R: Reporter,
{
    type TestCaseReporter = R::TestCaseReporter;

    fn test_run_starting(&mut self, tests: &[Test]) {
        (**self).test_run_starting(tests)
    }

    fn test_run_ended(&mut self, summary: &Summary) {
        (**self).test_run_ended(summary)
    }

    fn test_case_reporter(&mut self) -> Self::TestCaseReporter {
        (**self).test_case_reporter()
    }
}

/// The handler for events that occur during the execution of a test case.
pub trait TestCaseReporter {
    fn test_case_starting(&mut self);
    fn test_case_ended(&mut self, result: &TestCaseResult);
}

impl<T: ?Sized> TestCaseReporter for &mut T
where
    T: TestCaseReporter,
{
    fn test_case_starting(&mut self) {
        (**self).test_case_starting()
    }

    fn test_case_ended(&mut self, result: &TestCaseResult) {
        (**self).test_case_ended(result)
    }
}

impl<T: ?Sized> TestCaseReporter for Box<T>
where
    T: TestCaseReporter,
{
    fn test_case_starting(&mut self) {
        (**self).test_case_starting()
    }

    fn test_case_ended(&mut self, result: &TestCaseResult) {
        (**self).test_case_ended(result)
    }
}
