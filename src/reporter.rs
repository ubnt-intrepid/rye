#![allow(missing_docs)]

pub mod console;

use maybe_unwind::Unwind;
use std::fmt;

/// The handler for events that occur during the execution of a test case.
pub trait TestCaseReporter {
    fn test_case_starting(&mut self);
    fn test_case_ended(&mut self);

    fn section_starting(&mut self, name: Option<&str>);
    fn section_passed(&mut self, name: Option<&str>);
    fn section_failed(&mut self, name: Option<&str>, msg: &(dyn fmt::Debug + 'static));
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

    fn section_passed(&mut self, name: Option<&str>) {
        (**self).section_passed(name)
    }

    fn section_failed(&mut self, name: Option<&str>, msg: &(dyn fmt::Debug + 'static)) {
        (**self).section_failed(name, msg)
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

    fn section_passed(&mut self, name: Option<&str>) {
        (**self).section_passed(name)
    }

    fn section_failed(&mut self, name: Option<&str>, msg: &(dyn fmt::Debug + 'static)) {
        (**self).section_failed(name, msg)
    }

    fn section_terminated(&mut self, name: Option<&str>, unwind: &Unwind) {
        (**self).section_terminated(name, unwind)
    }
}
