//! Definition of command line interface.

use getopts::Options;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::{path::Path, str::FromStr};

/// Command line arguments.
#[derive(Debug)]
pub struct Args {
    list: bool,
    color: ColorConfig,
    globs: Option<GlobSet>,
}

impl Args {
    /// Parse command line arguments.
    pub fn from_env() -> Result<Self, ExitStatus> {
        let parser = Parser::new();
        match parser.parse_args() {
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

    pub(crate) fn list_tests(&self) -> bool {
        self.list
    }

    pub(crate) fn color(&self) -> &ColorConfig {
        &self.color
    }

    pub(crate) fn is_match(&self, name: &str) -> bool {
        self.globs
            .as_ref()
            .map_or(true, |globs| globs.is_match(name))
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

struct Parser {
    args: Vec<String>,
    opts: Options,
}

impl Parser {
    fn new() -> Self {
        let mut opts = Options::new();
        opts.optflag("h", "help", "Display this message (longer with --help)");
        opts.optflag("", "list", "List all tests and benchmarks");
        opts.optopt(
            "",
            "color",
            "Configure coloring of output:
                auto   = colorize if stdout is a tty and tests are run on serially (default);
                always = always colorize output;
                never  = never colorize output;",
            "auto|always|never",
        );

        // The following options and flags are reserved for keeping the compatibility with
        // the built-in test harness.
        opts.optflag("", "ignored", "");
        opts.optflag("", "test", "");
        opts.optflag("", "bench", "");
        opts.optflag("", "nocapture", "");
        opts.optflag("q", "quiet", "");
        opts.optflag("", "exact", "");
        opts.optopt("", "logfile", "", "PATH");
        opts.optopt("", "test-threads", "", "n_threads");
        opts.optopt("", "skip", "", "FILTER");
        opts.optopt("", "format", "", "");
        opts.optopt("Z", "", "", "unstable-options");

        Self {
            args: std::env::args().collect(),
            opts,
        }
    }

    fn print_usage(&self) {
        let binary = &self.args[0];
        let progname = Path::new(binary)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(binary);

        let message = format!("Usage: {} [OPTIONS] [PATTERN]..", progname);
        eprintln!(
            r#"{usage}
If the PATTERN strings are specified, the test cases with the matching name are executed,
otherwise all test cases are executed."#,
            usage = self.opts.usage(&message)
        );
    }

    fn parse_args(&self) -> Result<Option<Args>, Box<dyn std::error::Error>> {
        let args = &self.args[..];

        let matches = self.opts.parse(args.get(1..).unwrap_or(args))?;
        if matches.opt_present("h") {
            return Ok(None);
        }

        let list = matches.opt_present("list");
        let color = matches.opt_get("color")?.unwrap_or(ColorConfig::Auto);

        let globs = match &matches.free[..] {
            patterns if !patterns.is_empty() => {
                let mut globs = GlobSetBuilder::new();
                for pattern in patterns {
                    globs.add(Glob::new(pattern)?);
                }
                Some(globs.build()?)
            }
            _ => None,
        };

        Ok(Some(Args { list, color, globs }))
    }
}

/// Exit status code used as a result of the test process.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExitStatus(pub(crate) i32);

impl ExitStatus {
    pub(crate) const OK: Self = Self(0);
    pub(crate) const FAILED: Self = Self(101);

    /// Return the raw exit code.
    #[inline]
    pub fn code(self) -> i32 {
        self.0
    }

    /// Terminate the test process with the exit code.
    ///
    /// This method **should not** be called before the cleanup
    /// of the test process has completed.
    #[inline]
    pub fn exit(self) -> ! {
        std::process::exit(self.code());
    }
}
