use super::{Failure, Reporter, Status, Summary, TestCaseSummary};
use crate::{
    cli::args::{Args, ColorConfig},
    test::{Test, TestDesc},
};
use std::{
    fmt,
    io::{self, Write as _},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

struct Colored<T> {
    val: T,
    spec: Option<ColorSpec>,
}

impl<T> Colored<T> {
    fn fg(mut self, color: Color) -> Self {
        self.spec
            .get_or_insert_with(ColorSpec::new)
            .set_fg(Some(color));
        self
    }

    fn fmt_colored<W: ?Sized>(&self, w: &mut W) -> io::Result<()>
    where
        T: fmt::Display,
        W: WriteColor,
    {
        if let Some(ref spec) = self.spec {
            w.set_color(spec)?;
        }
        write!(w, "{}", &self.val)?;
        if let Some(..) = self.spec {
            w.reset()?;
        }
        Ok(())
    }
}

fn colored<T>(val: T) -> Colored<T> {
    Colored { val, spec: None }
}

pub struct ConsoleReporter {
    stream: StandardStream,
}

impl ConsoleReporter {
    pub fn new(args: &Args) -> Self {
        Self {
            stream: StandardStream::stdout(match args.color {
                ColorConfig::Auto => ColorChoice::Auto,
                ColorConfig::Always => ColorChoice::Always,
                ColorConfig::Never => ColorChoice::Never,
            }),
        }
    }

    fn print_test_case_summary(
        &self,
        w: &mut StandardStreamLock<'_>,
        summary: &TestCaseSummary,
    ) -> io::Result<()> {
        let status = match summary.status() {
            Status::Passed => colored("ok").fg(Color::Green),
            Status::Failed => colored("FAILED").fg(Color::Red),
        };
        write!(w, "test {} ... ", summary.desc.name(),)?;
        status.fmt_colored(w)?;
        writeln!(w)?;
        Ok(())
    }

    fn print_summary(&self, w: &mut StandardStreamLock<'_>, summary: &Summary) -> io::Result<()> {
        if !summary.failed.is_empty() {
            writeln!(w)?;
            writeln!(w, "failures:")?;
            for result in &summary.failed {
                writeln!(
                    w,
                    "---- {} at {} ----",
                    result.desc.name(),
                    result.desc.location
                )?;
                for failure in &result.failures {
                    match failure {
                        Failure::Unwind(ref unwind) => writeln!(w, "{}", unwind)?,
                        Failure::Error(ref err) => writeln!(w, "{}", &**err)?,
                    }
                }
                writeln!(w)?;
            }

            writeln!(w)?;
            writeln!(w, "failures:")?;
            for result in &summary.failed {
                writeln!(w, "    {}", result.desc.name())?;
            }
        }

        let status = if summary.is_passed() {
            colored("ok").fg(Color::Green)
        } else {
            colored("FAILED").fg(Color::Red)
        };
        writeln!(w)?;
        write!(w, "test result: ")?;
        status.fmt_colored(w)?;
        write!(w, ".")?;
        writeln!(
            w,
            " {passed} passed; {failed} failed; {filtered_out} filtered out",
            passed = summary.passed.len(),
            failed = summary.failed.len(),
            filtered_out = summary.filtered_out.len(),
        )?;

        Ok(())
    }
}

impl Reporter for ConsoleReporter {
    fn test_run_starting(&self, tests: &[Test]) {
        let mut w = self.stream.lock();

        let num_tests = tests.iter().filter(|test| !test.filtered_out).count();
        let _ = writeln!(w, "running {} tests", num_tests);
    }

    fn test_run_ended(&self, summary: &Summary) {
        let mut w = self.stream.lock();
        let _ = self.print_summary(&mut w, summary);
    }

    fn test_case_starting(&self, _: &TestDesc) {}

    fn test_case_ended(&self, summary: &TestCaseSummary) {
        let mut w = self.stream.lock();
        let _ = self.print_test_case_summary(&mut w, &summary);
    }
}
