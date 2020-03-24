#![allow(missing_docs)]

pub(crate) mod console;

use crate::{
    location::Location,
    test::{TestCase, TestDesc},
};

#[derive(Debug)]
pub(crate) enum Outcome {
    Passed,
    Errored(anyhow::Error),
    Skipped {
        location: &'static Location,
        reason: String,
    },
    Failed {
        location: &'static Location,
        reason: String,
    },
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct TestCaseSummary {
    pub(crate) desc: &'static TestDesc,
    pub(crate) outcome: Outcome,
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Summary {
    pub(crate) passed: Vec<TestCaseSummary>,
    pub(crate) failed: Vec<TestCaseSummary>,
    pub(crate) skipped: Vec<TestCaseSummary>,
    pub(crate) filtered_out: Vec<&'static TestDesc>,
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
        match result.outcome {
            Outcome::Passed => self.passed.push(result),
            Outcome::Errored(..) | Outcome::Failed { .. } => self.failed.push(result),
            Outcome::Skipped { .. } => self.skipped.push(result),
        }
    }
}

pub trait Reporter {
    fn test_run_starting(&self, tests: &[&dyn TestCase]);
    fn test_run_ended(&self, summary: &Summary);

    fn test_case_starting(&self, desc: &TestDesc);
    fn test_case_ended(&self, summary: &TestCaseSummary);
}

macro_rules! impl_reporter_body {
    () => {
        fn test_run_starting(&self, tests: &[&dyn TestCase]) {
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
