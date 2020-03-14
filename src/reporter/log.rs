#![allow(missing_docs)]

use super::{Reporter, Status, Summary, TestCaseSummary};
use crate::test::{Test, TestDesc};

#[derive(Debug, Clone)]
pub struct LogReporter {
    _p: (),
}

impl LogReporter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _p: () }
    }
}

impl Reporter for LogReporter {
    fn test_run_starting(&self, tests: &[Test]) {
        let suffix = match tests.len() {
            1 => "",
            _ => "s",
        };
        log::info!("running {} test{}", tests.len(), suffix);
    }

    fn test_run_ended(&self, summary: &Summary) {
        if summary.is_passed() {
            log::info!("test status: ok");
        } else {
            log::error!("test status: FAILED");
        }
    }

    fn test_case_starting(&self, desc: &TestDesc) {
        log::info!("start: {}", desc.name());
    }

    fn test_case_ended(&self, summary: &TestCaseSummary) {
        match summary.status() {
            Status::Passed => log::info!("{}: ok", summary.desc.name()),
            Status::Failed => log::error!("{}: FAILED", summary.desc.name()),
        }
    }
}
