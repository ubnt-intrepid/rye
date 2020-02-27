//! Definition of command line interface.

use crate::cli::exit_status::ExitStatus;
use getopts::Options;
use std::{path::Path, str::FromStr};

/// Command line arguments.
#[derive(Debug)]
pub struct Args {
    pub list_tests: bool,
    pub filter_pattern: Option<String>,
    pub filter_exact: bool,
    pub color: ColorConfig,
    pub skip_patterns: Vec<String>,
}

impl Args {
    /// Parse command line arguments.
    pub fn from_env() -> Result<Self, ExitStatus> {
        let args: Vec<_> = std::env::args().collect();
        let parser = Parser::new(&args[..]);
        match parser.parse() {
            Ok(Some(args)) => Ok(args),
            Ok(None) => {
                parser.print_usage();
                Err(ExitStatus::OK)
            }
            Err(err) => {
                eprintln!("CLI argument error: {}", err);
                Err(ExitStatus::FAILED)
            }
        }
    }

    pub(crate) fn is_filtered_out(&self, test_name: &str) -> bool {
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

/// The color configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ColorConfig {
    Auto,
    Always,
    Never,
}

impl FromStr for ColorConfig {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(ColorConfig::Auto),
            "always" => Ok(ColorConfig::Always),
            "never" => Ok(ColorConfig::Never),
            v => Err(format!(
                "argument for --color must be auto, always, or never (was {})",
                v
            )
            .into()),
        }
    }
}

struct Parser<'a> {
    args: &'a [String],
    opts: Options,
}

impl<'a> Parser<'a> {
    fn new(args: &'a [String]) -> Self {
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

        Self { args, opts }
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

    fn parse(&self) -> Result<Option<Args>, Box<dyn std::error::Error>> {
        let args = &self.args[..];

        let matches = self.opts.parse(args.get(1..).unwrap_or(args))?;
        if matches.opt_present("h") {
            return Ok(None);
        }

        let list_tests = matches.opt_present("list");
        let filter_exact = matches.opt_present("exact");
        let color = matches.opt_get("color")?.unwrap_or(ColorConfig::Auto);
        let skip_patterns = matches.opt_strs("skip");
        let filter_pattern = matches.free.get(0).cloned();

        Ok(Some(Args {
            list_tests,
            filter_pattern,
            filter_exact,
            color,
            skip_patterns,
        }))
    }
}
