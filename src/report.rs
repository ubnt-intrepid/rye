use crate::{
    cli::{Args, ColorConfig, ExitStatus, OutputFormat},
    test_case::TestDesc,
};
use console::{Style, StyledObject, Term};
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
    pub passed: Vec<TestDesc>,

    /// Failed test cases with the error messages.
    pub failed: Vec<(TestDesc, Option<Arc<Cow<'static, str>>>)>,

    /// Test cases skipped because they do not satisfy the execution conditions.
    pub ignored: Vec<TestDesc>,

    /// Test cases filtered out.
    pub filtered_out: Vec<TestDesc>,
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

    /// Return an iterator of skipped test cases.
    #[inline]
    #[allow(dead_code)]
    pub fn skipped(&self) -> impl Iterator<Item = (&TestDesc, &str)> + '_ {
        let ignored = self.ignored.iter().map(|desc| (desc, "ignored"));
        let filtered_out = self.filtered_out.iter().map(|desc| (desc, "filtered out"));
        ignored.chain(filtered_out)
    }
}

pub(crate) struct Printer {
    term: Term,
    format: OutputFormat,
    style: Style,
}

impl Printer {
    pub(crate) fn new(args: &Args) -> Self {
        Self {
            term: Term::buffered_stdout(),
            format: args.format,
            style: {
                let mut style = Style::new();
                match args.color {
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
        tests: impl IntoIterator<Item = impl std::ops::Deref<Target = TestDesc>>,
    ) -> io::Result<()> {
        let quiet = self.format == OutputFormat::Terse;

        let mut num_tests = 0;

        for test in tests {
            let desc = &*test;
            num_tests += 1;
            writeln!(&self.term, "{}: test", desc.name)?;
        }

        if !quiet {
            fn plural_suffix(n: usize) -> &'static str {
                match n {
                    1 => "",
                    _ => "s",
                }
            }

            if num_tests != 0 {
                writeln!(&self.term)?;
            }
            writeln!(
                &self.term,
                "{} test{}, 0 benchmark{}",
                num_tests,
                plural_suffix(num_tests),
                plural_suffix(0)
            )?;
        }

        Ok(())
    }

    pub(crate) fn print_result(
        &self,
        desc: &TestDesc,
        name_length: usize,
        outcome: Option<&Outcome>,
    ) -> io::Result<()> {
        match self.format {
            OutputFormat::Pretty => self.print_result_pretty(desc, name_length, outcome),
            OutputFormat::Terse => self.print_result_terse(desc, name_length, outcome),
        }
    }

    pub(crate) fn print_result_pretty(
        &self,
        desc: &TestDesc,
        name_length: usize,
        outcome: Option<&Outcome>,
    ) -> io::Result<()> {
        let name = desc.name;

        match outcome {
            Some(outcome) => match outcome.kind() {
                OutcomeKind::Passed => {
                    writeln!(
                        &self.term,
                        "test {0:<1$} ... {2}",
                        name,
                        name_length,
                        self.styled("ok").green()
                    )?;
                }
                OutcomeKind::Failed => {
                    writeln!(
                        &self.term,
                        "test {0:<1$} ... {2}",
                        name,
                        name_length,
                        self.styled("FAILED").red()
                    )?;
                }
            },
            None => {
                writeln!(
                    &self.term,
                    "test {0:<1$} ... {2}",
                    name,
                    name_length,
                    self.styled("ignored").yellow()
                )?;
            }
        }
        self.term.flush()
    }

    fn print_result_terse(
        &self,
        _: &TestDesc,
        _: usize,
        outcome: Option<&Outcome>,
    ) -> io::Result<()> {
        let ch = match outcome {
            Some(o) => match o.kind() {
                OutcomeKind::Passed => ".",
                OutcomeKind::Failed => "F",
            },
            None => "i",
        };
        self.term.write_str(ch)?;
        self.term.flush()
    }

    pub(crate) fn print_report(&self, report: &Report) -> io::Result<()> {
        let mut status = self.styled("ok").green();

        if !report.failed.is_empty() {
            status = self.styled("FAILED").red();
            writeln!(self.term())?;
            writeln!(self.term(), "failures:")?;
            for (desc, msg) in &report.failed {
                writeln!(self.term(), "---- {} ----", desc.name)?;
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
                writeln!(self.term(), "    {}", desc.name)?;
            }
        }

        writeln!(self.term())?;
        writeln!(self.term(), "test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {filtered_out} filtered out",
            status = status,
            passed = report.passed.len(),
            failed = report.failed.len(),
            ignored = report.ignored.len(),
            filtered_out = report.filtered_out.len(),
        )?;

        Ok(())
    }
}
