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

    /// Create an `Outcome` representing that the benchmark test was successfully run.
    pub fn measured(average: u64, variance: u64) -> Self {
        Self::new(OutcomeKind::Measured { average, variance })
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
    Measured { average: u64, variance: u64 },
}

/// A report on test suite execution.
#[derive(Debug)]
#[non_exhaustive]
pub struct Report {
    /// Passed test cases.
    pub passed: Vec<TestDesc>,

    /// Failed test cases with the error messages.
    pub failed: Vec<(TestDesc, Option<Arc<Cow<'static, str>>>)>,

    /// Benchmark results.
    pub measured: Vec<(TestDesc, (u64, u64))>,

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

    pub(crate) fn print(&self, printer: &Printer) -> io::Result<()> {
        let mut status = printer.styled("ok").green();

        if !self.failed.is_empty() {
            status = printer.styled("FAILED").red();
            writeln!(printer.term())?;
            writeln!(printer.term(), "failures:")?;
            for (desc, msg) in &self.failed {
                writeln!(printer.term(), "---- {} ----", desc.name)?;
                if let Some(msg) = msg {
                    write!(printer.term(), "{}", msg)?;
                    if msg.chars().last().map_or(true, |c| c != '\n') {
                        writeln!(printer.term())?;
                    }
                }
            }

            writeln!(printer.term())?;
            writeln!(printer.term(), "failures:")?;
            for (desc, _) in &self.failed {
                writeln!(printer.term(), "    {}", desc.name)?;
            }
        }

        writeln!(printer.term())?;
        writeln!(printer.term(), "test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out",
            status = status,
            passed = self.passed.len(),
            failed = self.failed.len(),
            ignored = self.ignored.len(),
            measured = self.measured.len(),
            filtered_out = self.filtered_out.len(),
        )?;

        Ok(())
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

    pub(crate) fn styled<D>(&self, val: D) -> StyledObject<D> {
        self.style.apply_to(val)
    }

    pub(crate) fn print_list(
        &self,
        tests: impl IntoIterator<Item = impl std::ops::Deref<Target = TestDesc>>,
    ) {
        let quiet = self.format == OutputFormat::Terse;

        let mut num_tests = 0;

        for test in tests {
            let desc = &*test;
            num_tests += 1;
            let _ = writeln!(&self.term, "{}: test", desc.name);
        }

        if !quiet {
            fn plural_suffix(n: usize) -> &'static str {
                match n {
                    1 => "",
                    _ => "s",
                }
            }

            if num_tests != 0 {
                let _ = writeln!(&self.term);
            }
            let _ = writeln!(
                &self.term,
                "{} test{}, 0 benchmark{}",
                num_tests,
                plural_suffix(num_tests),
                plural_suffix(0)
            );
        }
    }

    pub(crate) fn print_result(
        &self,
        desc: &TestDesc,
        name_length: usize,
        outcome: Option<&Outcome>,
    ) {
        match self.format {
            OutputFormat::Pretty => self.print_result_pretty(desc, name_length, outcome),
            OutputFormat::Terse => self.print_result_terse(desc, name_length, outcome),
            OutputFormat::Json => eprintln!(
                "{warning}: JSON format is not supported",
                warning = self.styled("warning").yellow()
            ),
        }
    }

    fn print_result_pretty(&self, desc: &TestDesc, name_length: usize, outcome: Option<&Outcome>) {
        let name = desc.name;

        match outcome {
            Some(outcome) => match outcome.kind() {
                OutcomeKind::Passed => {
                    let _ = writeln!(
                        &self.term,
                        "test {0:<1$} ... {2}",
                        name,
                        name_length,
                        self.styled("ok").green()
                    );
                }
                OutcomeKind::Failed => {
                    let _ = writeln!(
                        &self.term,
                        "test {0:<1$} ... {2}",
                        name,
                        name_length,
                        self.styled("FAILED").red()
                    );
                }
                OutcomeKind::Measured { average, variance } => {
                    let _ = writeln!(
                        &self.term,
                        "test {0:<1$} ... {2}: {3:>11} ns/iter (+/- {4})",
                        name,
                        name_length,
                        self.styled("bench").cyan(),
                        average,
                        variance
                    );
                }
            },
            None => {
                let _ = writeln!(
                    &self.term,
                    "test {0:<1$} ... {2}",
                    name,
                    name_length,
                    self.styled("ignored").yellow()
                );
            }
        }
        let _ = self.term.flush();
    }

    fn print_result_terse(&self, desc: &TestDesc, name_length: usize, outcome: Option<&Outcome>) {
        let ch = match outcome {
            Some(o) => match o.kind() {
                OutcomeKind::Passed => ".",
                OutcomeKind::Failed => "F",
                OutcomeKind::Measured { .. } => {
                    // benchmark test does not support terse format.
                    return self.print_result_pretty(desc, name_length, outcome);
                }
            },
            None => "i",
        };
        let _ = self.term.write_str(ch);
        let _ = self.term.flush();
    }
}
