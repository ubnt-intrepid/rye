mod coverage;
mod doc;
mod hook;
mod lint;
mod shell;
mod test;

use crate::shell::Shell;
use pico_args::Arguments;

fn main() -> anyhow::Result<()> {
    let show_help = || {
        eprintln!(
            "\
cargo-xtask
Free style automation tool

Usage:
cargo xtask <SUBCOMMAND>

Subcommands:
    test            Run test
    coverage        Run coverage test
    doc             Generate API docs
    lint            Run lints
    install-hooks   Install Git hooks
    pre-commit      Run Git pre-commit hook
"
        );
    };

    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        show_help();
        return Ok(());
    }

    let subcommand = args.subcommand().map_err(|err| {
        show_help();
        err
    })?;

    match subcommand.as_deref() {
        Some("test") => {
            let sh = Shell::new();
            crate::test::do_test(&sh)
        }
        Some("doc") => {
            let serve_addr = if args.contains(["-s", "--serve"]) {
                let addr = args //
                    .opt_value_from_str(["-b", "--bind"])?
                    .unwrap_or_else(|| ([0, 0, 0, 0], 8000).into());
                Some(addr)
            } else {
                None
            };
            let sh = Shell::new();
            crate::doc::do_doc(&sh, serve_addr)
        }
        Some("coverage") => {
            let sh = Shell::new();
            crate::coverage::do_coverage(&sh)
        }
        Some("lint") => {
            let sh = Shell::new();
            lint::do_lint(&sh)
        }
        Some("install-hooks") => {
            let sh = Shell::new();
            crate::hook::install(&sh)
        }
        Some("pre-commit") => {
            let sh = Shell::new();
            crate::hook::pre_commit(&sh)
        }
        Some(s) => {
            show_help();
            anyhow::bail!("invalid subcommand: {}", s);
        }
        None => {
            show_help();
            anyhow::bail!("missing subcommand");
        }
    }
}
