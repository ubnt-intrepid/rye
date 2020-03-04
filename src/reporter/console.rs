use super::TestCaseReporter;
use crate::{
    cli::{
        args::{Args, ColorConfig},
        exit_status::ExitStatus,
    },
    test::{TestDesc, TestResult},
};
use console::{Style, StyledObject, Term};
use futures::channel::oneshot;
use maybe_unwind::Unwind;
use std::{
    borrow::Cow,
    collections::HashMap,
    error, fmt,
    io::{self, Write},
    sync::Arc,
};

/// The outcome of performing a test.
#[derive(Debug)]
pub(crate) struct Outcome {
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
pub(crate) struct Report {
    /// Passed test cases.
    pub passed: Vec<&'static TestDesc>,

    /// Failed test cases with the error messages.
    pub failed: Vec<(&'static TestDesc, Option<Arc<Cow<'static, str>>>)>,

    /// Test cases filtered out.
    pub filtered_out: Vec<&'static TestDesc>,
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

pub struct ConsoleReporter {
    term: Term,
    style: Style,
}

impl ConsoleReporter {
    pub fn new(args: &Args) -> Self {
        Self {
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
        tests: impl IntoIterator<Item = &'static TestDesc>,
    ) -> io::Result<()> {
        let term = io::stdout();
        let mut term = term.lock();
        let mut num_tests = 0;

        for test in tests {
            let test = &*test;
            num_tests += 1;
            writeln!(term, "{}: test", test.name())?;
        }

        fn plural_suffix(n: usize) -> &'static str {
            match n {
                1 => "",
                _ => "s",
            }
        }

        if num_tests != 0 {
            writeln!(term)?;
        }
        writeln!(term, "{} test{}", num_tests, plural_suffix(num_tests),)?;

        term.flush()?;
        Ok(())
    }

    pub(crate) fn print_result(
        &self,
        test: &TestDesc,
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
            test.name(),
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
            for (test, msg) in &report.failed {
                writeln!(self.term(), "---- {} ----", test.name())?;
                if let Some(msg) = msg {
                    write!(self.term(), "{}", msg)?;
                    if msg.chars().last().map_or(true, |c| c != '\n') {
                        writeln!(self.term())?;
                    }
                }
            }

            writeln!(self.term())?;
            writeln!(self.term(), "failures:")?;
            for (test, _) in &report.failed {
                writeln!(self.term(), "    {}", test.name())?;
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

#[derive(Debug)]
pub(crate) struct ConsoleTestCaseReporter {
    tx: Option<oneshot::Sender<Result<(), ErrorMessage>>>,
    failures: HashMap<String, String>,
}

impl ConsoleTestCaseReporter {
    pub(crate) fn new(tx: futures::channel::oneshot::Sender<Result<(), ErrorMessage>>) -> Self {
        Self {
            tx: Some(tx),
            failures: HashMap::new(),
        }
    }

    fn make_outcome(&mut self) -> Result<(), ErrorMessage> {
        if self.failures.is_empty() {
            Ok(())
        } else {
            Err(ErrorMessage(format!("{:?}", self).into()))
        }
    }
}

impl TestCaseReporter for ConsoleTestCaseReporter {
    fn test_case_starting(&mut self) {}

    fn test_case_ended(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(self.make_outcome());
        }
    }

    fn section_starting(&mut self, _: Option<&str>) {}

    fn section_ended(&mut self, name: Option<&str>, result: &dyn TestResult) {
        if !result.is_success() {
            let name = name.unwrap_or("__root__");
            self.failures.insert(
                name.into(),
                result
                    .error_message()
                    .map_or("<unknown>".into(), |msg| format!("{:?}", msg)),
            );
        }
    }

    fn section_terminated(&mut self, name: Option<&str>, unwind: &Unwind) {
        let name = name.unwrap_or("__root__");
        self.failures.insert(name.into(), unwind.to_string());
    }
}

pub(crate) struct ErrorMessage(Box<dyn error::Error + Send + Sync>);

impl fmt::Debug for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            fmt::Debug::fmt(&*self.0, f)
        } else {
            fmt::Display::fmt(&*self.0, f)
        }
    }
}
