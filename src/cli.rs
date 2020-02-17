//! Definition of command line interface.

use getopts::Options;
use std::{path::Path, str::FromStr};

/// Command line arguments.
#[derive(Debug)]
#[non_exhaustive]
pub struct Args {
    pub list: bool,
    pub filter: Option<String>,
    pub filter_exact: bool,
    pub run_ignored: bool,
    pub color: ColorConfig,
    pub format: OutputFormat,
    pub skip: Vec<String>,
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

    pub(crate) fn is_filtered(&self, name: &str) -> bool {
        if let Some(ref filter) = self.filter {
            if self.filter_exact && name != filter {
                return true;
            }
            if !name.contains(filter) {
                return true;
            }
        }

        for skip_filter in &self.skip {
            if self.filter_exact && name != skip_filter {
                return true;
            }
            if !name.contains(skip_filter) {
                return true;
            }
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

/// The output format.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputFormat {
    Pretty,
    Terse,
}

impl FromStr for OutputFormat {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pretty" => Ok(OutputFormat::Pretty),
            "terse" => Ok(OutputFormat::Terse),
            s => Err(format!(
                "argument for --format must be pretty, terse, or json (was {})",
                s
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
        opts.optflag("", "ignored", "Run only ignored tests");
        opts.optmulti(
            "",
            "skip",
            "Skip tests whose names contain FILTER (this flag can be used multiple times)",
            "FILTER",
        );
        opts.optflag(
            "q",
            "quiet",
            "Display one character per test instead of one line. Alias to --format=terse",
        );
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
        opts.optopt(
            "",
            "format",
            "Configure formatting of output:
                pretty = Print verbose output;
                terse  = Display one character per test",
            "pretty|terse|json",
        );

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

        let message = format!("Usage: {} [OPTIONS] [FILTER]", progname);
        eprintln!(
            r#"{usage}
    
    The FILTER string is tested against the name of all tests, and only those
    tests whose names contain the filter are run."#,
            usage = self.opts.usage(&message)
        );
    }

    fn parse_args(&self) -> Result<Option<Args>, Box<dyn std::error::Error>> {
        let args = &self.args[..];

        let matches = self.opts.parse(args.get(1..).unwrap_or(args))?;
        if matches.opt_present("h") {
            return Ok(None);
        }

        let filter = matches.free.get(0).cloned();
        let run_ignored = matches.opt_present("ignored");
        let quiet = matches.opt_present("quiet");
        let filter_exact = matches.opt_present("exact");
        let list = matches.opt_present("list");

        let color = matches.opt_get("color")?.unwrap_or(ColorConfig::Auto);

        let format = matches.opt_get("format")?.unwrap_or_else(|| {
            if quiet {
                OutputFormat::Terse
            } else {
                OutputFormat::Pretty
            }
        });

        let skip = matches.opt_strs("skip");

        Ok(Some(Args {
            list,
            filter,
            filter_exact,
            run_ignored,
            color,
            format,
            skip,
        }))
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
