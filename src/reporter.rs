#![allow(missing_docs)]

mod console;
mod log;

pub use self::{console::ConsoleReporter, log::LogReporter};

use crate::{
    runner::result::{Summary, TestCaseResult},
    test::{Test, TestDesc},
};

pub trait Reporter {
    fn test_run_starting(&self, tests: &[Test]);
    fn test_run_ended(&self, summary: &Summary);

    fn test_case_starting(&self, desc: &TestDesc);
    fn test_case_ended(&self, result: &TestCaseResult);
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

        fn test_case_ended(&self, result: &TestCaseResult) {
            (**self).test_case_ended(result)
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
