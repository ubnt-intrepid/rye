#![allow(missing_docs)]

use crate::{
    reporter::{Outcome, Reporter, Summary, TestCaseSummary},
    test::{TestCase, TestDesc},
};

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
    fn test_run_starting(&self, tests: &[&dyn TestCase]) {
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
        match summary.outcome {
            Outcome::Passed => log::info!("{}: ok", summary.desc.name()),
            Outcome::Errored(..) | Outcome::Panicked { .. } => {
                log::error!("{}: FAILED", summary.desc.name())
            }
            Outcome::Skipped { .. } => log::info!("{}: skipped", summary.desc.name()),
        }
    }
}
