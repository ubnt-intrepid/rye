use super::{FailKind, Reporter, ResultDisposition, Summary, TestCaseSummary};
use crate::{
    cli::args::{Args, ColorConfig},
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

    fn print_test_case_summary(&self, summary: &TestCaseSummary) -> io::Result<()> {
        let status = match summary.result {
            ResultDisposition::Passed => self.styled("ok").green(),
            ResultDisposition::Failed if summary.desc.todo => self.styled("FAILED (todo)").yellow(),
            ResultDisposition::Failed => self.styled("FAILED").red(),
        };
        writeln!(
            &self.term,
            "test {0:<1$} ... {2}",
            summary.desc.name(),
            self.name_length,
            status
        )?;
        self.term.flush()
    }

    fn print_summary(&mut self, summary: &Summary) -> io::Result<()> {
        let status = if summary.is_passed() {
            self.styled("ok").green()
        } else {
            self.styled("FAILED").red()
        };

        if !summary.failed.is_empty() {
            writeln!(&self.term)?;
            writeln!(&self.term, "failures:")?;
            for result in &summary.failed {
                writeln!(&self.term, "---- {} ----", result.desc.name())?;
                for (i, fail) in result.fails.iter().enumerate() {
                    if i > 0 {
                        writeln!(&self.term)?;
                    }
                    match fail.0 {
                        FailKind::Unwind(ref unwind) => writeln!(&self.term, "{}", unwind)?,
                        FailKind::Error(ref err) => writeln!(&self.term, "{}", &**err)?,
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
            "test result: {status}. {passed} passed; {failed} failed ({todo} todo); {filtered_out} filtered out",
            status = status,
            passed = summary.passed.len(),
            failed = summary.failed.len(),
            todo = summary.failed.iter().filter(|s| s.desc.todo).count(),
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

    fn test_case_ended(&self, summary: &TestCaseSummary) {
        let _ = self.inner.lock().unwrap().print_test_case_summary(&summary);
    }
}
