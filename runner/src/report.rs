use crate::cli::{Args, ColorConfig, ExitStatus};
use console::{Style, StyledObject, Term};
use rye::Test;
use std::{
    borrow::Cow,
    io::{self, Write},
    sync::Arc,
};

/// The outcome of performing a test.
#[derive(Debug)]
pub struct Outcome {
    kind: OutcomeKind,
    err_msg: Option<Arc<Cow<'static, str>>>,
}

impl Outcome {
    #[inline]
    fn new(kind: OutcomeKind) -> Self {
        Self {
            kind,
            err_msg: None,
        }
    }

    /// Create an `Outcome` representing that the test passed.
    #[inline]
    pub fn passed() -> Self {
        Self::new(OutcomeKind::Passed)
    }

    /// Create an `Outcome` representing that the test or benchmark failed.
    pub fn failed() -> Self {
        Self::new(OutcomeKind::Failed)
    }

    /// Specify the error message.
    pub fn error_message(self, err_msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            err_msg: Some(Arc::new(err_msg.into())),
            ..self
        }
    }

    pub(crate) fn kind(&self) -> &OutcomeKind {
        &self.kind
    }

    pub(crate) fn err_msg(&self) -> Option<Arc<Cow<'static, str>>> {
        self.err_msg.clone()
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum OutcomeKind {
    Passed,
    Failed,
}

/// A report on test suite execution.
#[derive(Debug)]
#[non_exhaustive]
pub struct Report {
    /// Passed test cases.
    pub passed: Vec<Test>,

    /// Failed test cases with the error messages.
    pub failed: Vec<(Test, Option<Arc<Cow<'static, str>>>)>,

    /// Test cases filtered out.
    pub filtered_out: Vec<Test>,
}

impl Report {
    /// Return an exit status used as a result of the test process.
    pub fn status(&self) -> ExitStatus {
        if self.failed.is_empty() {
            ExitStatus::OK
        } else {
            ExitStatus::FAILED
        }
    }
}

pub(crate) struct Printer {
    term: Term,
    style: Style,
}

impl Printer {
    pub(crate) fn new(args: &Args) -> Self {
        Self {
            term: Term::buffered_stdout(),
            style: {
                let mut style = Style::new();
                match args.color() {
                    ColorConfig::Always => style = style.force_styling(true),
                    ColorConfig::Never => style = style.force_styling(false),
                    _ => (),
                }
                style
            },
        }
    }

    pub(crate) fn term(&self) -> &Term {
        &self.term
    }

    fn styled<D>(&self, val: D) -> StyledObject<D> {
        self.style.apply_to(val)
    }

    pub(crate) fn print_list(
        &self,
        tests: impl IntoIterator<Item = impl std::ops::Deref<Target = Test>>,
    ) -> io::Result<()> {
        let mut num_tests = 0;

        for test in tests {
            let desc = &*test;
            num_tests += 1;
            writeln!(&self.term, "{}: test", desc.name())?;
        }

        fn plural_suffix(n: usize) -> &'static str {
            match n {
                1 => "",
                _ => "s",
            }
        }

        if num_tests != 0 {
            writeln!(&self.term)?;
        }
        writeln!(&self.term, "{} test{}", num_tests, plural_suffix(num_tests),)?;

        Ok(())
    }

    pub(crate) fn print_result(
        &self,
        desc: &Test,
        name_length: usize,
        outcome: &Outcome,
    ) -> io::Result<()> {
        let result = match outcome.kind() {
            OutcomeKind::Passed => self.styled("ok").green(),
            OutcomeKind::Failed => self.styled("FAILED").red(),
        };
        writeln!(
            &self.term,
            "test {0:<1$} ... {2}",
            desc.name(),
            name_length,
            result
        )?;
        self.term.flush()
    }

    pub(crate) fn print_report(&self, report: &Report) -> io::Result<()> {
        let mut status = self.styled("ok").green();

        if !report.failed.is_empty() {
            status = self.styled("FAILED").red();
            writeln!(self.term())?;
            writeln!(self.term(), "failures:")?;
            for (desc, msg) in &report.failed {
                writeln!(self.term(), "---- {} ----", desc.name())?;
                if let Some(msg) = msg {
                    write!(self.term(), "{}", msg)?;
                    if msg.chars().last().map_or(true, |c| c != '\n') {
                        writeln!(self.term())?;
                    }
                }
            }

            writeln!(self.term())?;
            writeln!(self.term(), "failures:")?;
            for (desc, _) in &report.failed {
                writeln!(self.term(), "    {}", desc.name())?;
            }
        }

        writeln!(self.term())?;
        writeln!(
            self.term(),
            "test result: {status}. {passed} passed; {failed} failed; {filtered_out} filtered out",
            status = status,
            passed = report.passed.len(),
            failed = report.failed.len(),
            filtered_out = report.filtered_out.len(),
        )?;

        Ok(())
    }
}
