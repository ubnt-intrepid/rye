use super::{
    args::{Args, ColorConfig, OutputFormat},
    outcome::{Outcome, OutcomeKind},
};
use crate::test_case::TestDesc;
use console::{Style, StyledObject, Term};
use std::io::Write;

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
        let quiet = self.format == super::args::OutputFormat::Terse;

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
