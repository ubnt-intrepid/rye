#![allow(missing_docs)]

use crate::test::TestDesc;

#[derive(Debug)]
pub enum TestResult {
    Passed,
    Failed,
}

#[derive(Debug)]
pub struct TestCaseResult {
    pub(crate) desc: &'static TestDesc,
    pub(crate) result: TestResult,
    pub(crate) error_message: Option<String>,
}

impl TestCaseResult {
    pub(crate) fn is_success(&self) -> bool {
        match self.result {
            TestResult::Passed => true,
            _ => false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Summary {
    pub(crate) passed: Vec<TestCaseResult>,
    pub(crate) failed: Vec<TestCaseResult>,
    pub(crate) filtered_out: Vec<&'static TestDesc>,
}

impl Summary {
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn append(&mut self, result: TestCaseResult) {
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
