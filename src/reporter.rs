#![allow(missing_docs)]

use crate::test::TestResult;
use maybe_unwind::Unwind;

/// The handler for events that occur during the execution of a test case.
pub trait TestCaseReporter {
    fn test_case_starting(&mut self);
    fn test_case_ended(&mut self);

    fn section_starting(&mut self, name: Option<&str>);
    fn section_ended(&mut self, name: Option<&str>, result: &dyn TestResult);
    fn section_terminated(&mut self, name: Option<&str>, unwind: &Unwind);
}

impl<E: ?Sized> TestCaseReporter for &mut E
where
    E: TestCaseReporter,
{
    fn test_case_starting(&mut self) {
        (**self).test_case_starting()
    }

    fn test_case_ended(&mut self) {
        (**self).test_case_ended()
    }

    fn section_starting(&mut self, name: Option<&str>) {
        (**self).section_starting(name)
    }

    fn section_ended(&mut self, name: Option<&str>, result: &dyn TestResult) {
        (**self).section_ended(name, result)
    }

    fn section_terminated(&mut self, name: Option<&str>, unwind: &Unwind) {
        (**self).section_terminated(name, unwind)
    }
}

impl<E: ?Sized> TestCaseReporter for Box<E>
where
    E: TestCaseReporter,
{
    fn test_case_starting(&mut self) {
        (**self).test_case_starting()
    }

    fn test_case_ended(&mut self) {
        (**self).test_case_ended()
    }

    fn section_starting(&mut self, name: Option<&str>) {
        (**self).section_starting(name)
    }

    fn section_ended(&mut self, name: Option<&str>, result: &dyn TestResult) {
        (**self).section_ended(name, result)
    }

    fn section_terminated(&mut self, name: Option<&str>, unwind: &Unwind) {
        (**self).section_terminated(name, unwind)
    }
}
