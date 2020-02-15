#![allow(dead_code)]

mod args;
mod context;
mod driver;
mod exit_status;
mod outcome;
mod printer;
mod report;

pub(crate) use context::TestContext;

use crate::test_case::TestCase;
use std::sync::Once;
use {args::Args, driver::TestDriver};

pub struct TestSuite<'a> {
    test_cases: &'a mut Vec<TestCase>,
}

impl TestSuite<'_> {
    #[doc(hidden)] // private API
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }
}

pub fn run_tests(tests: &[&dyn Fn(&mut TestSuite<'_>)]) {
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        maybe_unwind::set_hook();
    });

    let mut test_cases = vec![];
    for &test in tests {
        test(&mut TestSuite {
            test_cases: &mut test_cases,
        });
    }

    let mut driver = TestDriver::new(&args);
    let st = match futures::executor::block_on(driver.run_tests(test_cases)) {
        Ok(report) => report.status(),
        Err(status) => status,
    };
    st.exit();
}
