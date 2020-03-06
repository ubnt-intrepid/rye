use super::Reporter;
use crate::{
    cli::args::{Args, ColorConfig},
    runner::{Summary, TestCaseResult},
    test::{Test, TestDesc},
};
use console::{Style, StyledObject, Term};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct ConsoleReporter {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    term: Term,
    style: Style,
    name_length: usize,
}

impl Inner {
    fn styled<D>(&self, val: D) -> StyledObject<D> {
        self.style.apply_to(val)
    }

    fn print_result(&self, result: &TestCaseResult) -> io::Result<()> {
        let status = match result.result {
            crate::runner::result::TestResult::Passed => self.styled("ok").green(),
            crate::runner::result::TestResult::Failed => self.styled("FAILED").red(),
        };
        writeln!(
            &self.term,
            "test {0:<1$} ... {2}",
            result.desc.name(),
            self.name_length,
            status
        )?;
        self.term.flush()
    }

    fn print_summary(&mut self, summary: &Summary) -> io::Result<()> {
        let mut status = self.styled("ok").green();

        if !summary.failed.is_empty() {
            status = self.styled("FAILED").red();
            writeln!(&self.term)?;
            writeln!(&self.term, "failures:")?;
            for result in &summary.failed {
                writeln!(&self.term, "---- {} ----", result.desc.name())?;
                if let Some(ref msg) = result.error_message {
                    write!(&self.term, "{}", msg)?;
                    if msg.chars().last().map_or(true, |c| c != '\n') {
                        writeln!(&self.term)?;
                    }
                }
            }

            writeln!(&self.term)?;
            writeln!(&self.term, "failures:")?;
            for result in &summary.failed {
                writeln!(&self.term, "    {}", result.desc.name())?;
            }
        }

        writeln!(&self.term)?;
        writeln!(
            &self.term,
            "test result: {status}. {passed} passed; {failed} failed; {filtered_out} filtered out",
            status = status,
            passed = summary.passed.len(),
            failed = summary.failed.len(),
            filtered_out = summary.filtered_out.len(),
        )?;

        Ok(())
    }
}

impl ConsoleReporter {
    pub fn new(args: &Args) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                term: Term::buffered_stdout(),
                style: {
                    let mut style = Style::new();
                    match args.color {
                        ColorConfig::Always => style = style.force_styling(true),
                        ColorConfig::Never => style = style.force_styling(false),
                        _ => (),
                    }
                    style
                },
                name_length: 0,
            })),
        }
    }
}

impl Reporter for ConsoleReporter {
    fn test_run_starting(&self, tests: &[Test]) {
        let mut inner = self.inner.lock().unwrap();
        let _ = writeln!(&inner.term, "running {} tests", tests.len());
        inner.name_length = tests
            .iter()
            .map(|test| test.desc().name().len())
            .max()
            .unwrap_or(0);
    }

    fn test_run_ended(&self, summary: &Summary) {
        let _ = self.inner.lock().unwrap().print_summary(summary);
    }

    fn test_case_starting(&self, _: &TestDesc) {}

    fn test_case_ended(&self, result: &TestCaseResult) {
        let _ = self.inner.lock().unwrap().print_result(&result);
    }
}