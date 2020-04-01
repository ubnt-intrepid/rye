#![allow(missing_docs)]

use crate::{
    report::{Outcome, Reporter, Summary, TestCaseSummary},
    runtime::Spawner,
    test::{TestCase, TestDesc},
};
use getopts::Options;
use std::{
    collections::HashSet,
    fmt,
    io::{self, Write as _},
    path::Path,
    str::FromStr,
    sync::Arc,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

/// Command line arguments.
#[derive(Debug)]
struct Args {
    show_help: bool,
    list_tests: bool,
    filter_pattern: Option<String>,
    filter_exact: bool,
    color: ColorConfig,
    skip_patterns: Vec<String>,
}

impl Args {
    fn is_filtered_out(&self, test_name: &str) -> bool {
        let matches_filter = |pat: &str| {
            if self.filter_exact {
                test_name == pat
            } else {
                test_name.contains(pat)
            }
        };

        if self
            .filter_pattern
            .as_ref()
            .map_or(false, |pat| !matches_filter(pat))
        {
            return true;
        }

        if self.skip_patterns.iter().any(|pat| matches_filter(pat)) {
            return true;
        }

        false
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ColorConfig {
    Auto,
    Always,
    Never,
}

impl FromStr for ColorConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(ColorConfig::Auto),
            "always" => Ok(ColorConfig::Always),
            "never" => Ok(ColorConfig::Never),
            v => Err(anyhow::anyhow!(
                "argument for --color must be auto, always, or never (was {})",
                v
            )),
        }
    }
}

struct Parser {
    args: Vec<String>,
    opts: Options,
}

impl Parser {
    fn new(args: impl IntoIterator<Item = String>) -> Self {
        let mut opts = Options::new();
        opts.optflag("h", "help", "Display this message (longer with --help)");
        opts.optflag("", "list", "List all tests and benchmarks");
        opts.optflag(
            "",
            "exact",
            "Exactly match filters rather than by substring",
        );
        opts.optopt(
            "",
            "color",
            "Configure coloring of output:
                auto   = colorize if stdout is a tty and tests are run on serially (default);
                always = always colorize output;
                never  = never colorize output;",
            "auto|always|never",
        );
        opts.optmulti(
            "",
            "skip",
            "Skip tests whose names contain FILTER (this flag can be used multiple times)",
            "FILTER",
        );

        // The following options and flags are reserved for keeping the compatibility with
        // the built-in test harness.
        opts.optflag("", "ignored", "");
        opts.optflag("", "test", "");
        opts.optflag("", "bench", "");
        opts.optflag("", "nocapture", "");
        opts.optflag("q", "quiet", "");
        opts.optopt("", "logfile", "", "PATH");
        opts.optopt("", "test-threads", "", "n_threads");
        opts.optopt("", "format", "", "");
        opts.optopt("Z", "", "", "unstable-options");

        Self {
            args: args.into_iter().collect(),
            opts,
        }
    }

    fn print_usage(&self) {
        let binary = &self.args[0];
        let progname = Path::new(binary)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(binary);

        let message = format!("Usage: {} [OPTIONS] [FILTER]", progname);
        eprintln!(
            r#"{usage}
The FILTER string is tested against the name of all tests, and only those
tests whose names contain the filter are run."#,
            usage = self.opts.usage(&message)
        );
    }

    fn parse(&self) -> anyhow::Result<Args> {
        let args = &self.args[..];

        let matches = self.opts.parse(args.get(1..).unwrap_or(args))?;

        let show_help = matches.opt_present("help");
        let list_tests = matches.opt_present("list");
        let filter_exact = matches.opt_present("exact");
        let color = matches.opt_get("color")?.unwrap_or(ColorConfig::Auto);
        let skip_patterns = matches.opt_strs("skip");
        let filter_pattern = matches.free.get(0).cloned();

        Ok(Args {
            show_help,
            list_tests,
            filter_pattern,
            filter_exact,
            color,
            skip_patterns,
        })
    }
}

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

struct ConsoleReporter {
    stream: StandardStream,
}

impl ConsoleReporter {
    fn new(choice: ColorChoice) -> Self {
        Self {
            stream: StandardStream::stdout(choice),
        }
    }

    fn print_test_case_summary(
        &self,
        w: &mut StandardStreamLock<'_>,
        summary: &TestCaseSummary,
    ) -> io::Result<()> {
        let status = match summary.outcome {
            Outcome::Passed => colored("ok").fg(Color::Green),
            Outcome::Errored(..) | Outcome::Failed { .. } | Outcome::Panicked(..) => {
                colored("FAILED").fg(Color::Red)
            }
            Outcome::Skipped { .. } => colored("skipped").fg(Color::Yellow),
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

                match result.outcome {
                    Outcome::Errored(ref err) => {
                        writeln!(w, "{:?}", err)?;
                    }
                    Outcome::Panicked(ref unwind) => {
                        writeln!(w, "{:#}", unwind)?;
                    }
                    Outcome::Failed {
                        ref location,
                        ref reason,
                    } => {
                        writeln!(w, "{} {}", location, reason)?;
                    }
                    _ => unreachable!(),
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
            " {passed} passed; {failed} failed; {skipped} skipped; {filtered_out} filtered out",
            passed = summary.passed.len(),
            failed = summary.failed.len(),
            skipped = summary.skipped.len(),
            filtered_out = summary.filtered_out.len(),
        )?;

        Ok(())
    }
}

impl Reporter for ConsoleReporter {
    fn test_run_starting(&self, tests: &[&dyn TestCase]) {
        let mut w = self.stream.lock();
        let _ = writeln!(w, "running {} tests", tests.len());
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

pub struct SessionInner<'a> {
    parser: Parser,
    test_cases: &'a [&'a dyn TestCase],
}

impl<'a> SessionInner<'a> {
    #[inline]
    pub(crate) fn new(test_cases: &'a [&'a dyn TestCase]) -> Self {
        Self {
            parser: Parser::new(std::env::args()),
            test_cases,
        }
    }

    #[inline]
    pub fn session<'sess>(&'sess mut self, spawner: &'sess mut dyn Spawner) -> Session<'sess> {
        Session {
            parser: &mut self.parser,
            test_cases: self.test_cases,
            spawner,
        }
    }
}

pub struct Session<'sess> {
    parser: &'sess mut Parser,
    test_cases: &'sess [&'sess dyn TestCase],
    spawner: &'sess mut dyn Spawner,
}

impl Session<'_> {
    #[inline]
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let args = self.parser.parse()?;
        if args.show_help {
            self.parser.print_usage();
            return Ok(());
        }

        let mut registered_tests = vec![];
        let mut filtered_out_tests = vec![];
        let mut unique_test_names = HashSet::new();
        for test in self.test_cases {
            let desc = test.desc();
            let filtered_out = args.is_filtered_out(desc.name());

            anyhow::ensure!(
                unique_test_names.insert(desc.name().to_owned()),
                "the test name '{}' is conflicted",
                desc.name()
            );

            if filtered_out {
                filtered_out_tests.push(desc);
            } else {
                registered_tests.push(*test);
            }
        }

        // sort test cases by name.
        registered_tests.sort_by(|t1, t2| t1.desc().name().cmp(t2.desc().name()));

        if args.list_tests {
            let mut num_tests = 0;
            for test in &registered_tests {
                num_tests += 1;
                println!("{}: test", test.desc().name());
            }

            fn plural_suffix(n: usize) -> &'static str {
                match n {
                    1 => "",
                    _ => "s",
                }
            }

            if num_tests != 0 {
                println!();
            }
            println!("{} test{}", num_tests, plural_suffix(num_tests));

            return Ok(());
        }

        let reporter = Arc::new(ConsoleReporter::new(match args.color {
            ColorConfig::Auto => ColorChoice::Auto,
            ColorConfig::Always => ColorChoice::Always,
            ColorConfig::Never => ColorChoice::Never,
        }));

        reporter.test_run_starting(&registered_tests[..]);

        let mut summary = Summary::empty();
        summary.filtered_out.extend(filtered_out_tests);
        let mut handles = vec![];
        for test in registered_tests.drain(..) {
            let reporter = reporter.clone();
            let handle = test.spawn(&mut *self.spawner, reporter)?;
            handles.push(handle);
        }
        let results = futures_util::future::join_all(handles).await;
        for result in results {
            summary.append(result);
        }

        reporter.test_run_ended(&summary);

        if summary.is_passed() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(""))
        }
    }
}
