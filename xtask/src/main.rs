mod coverage;
mod doc;
mod env;
mod hook;
mod lint;
mod test;

use crate::env::Env;
use pico_args::Arguments;

fn main() -> anyhow::Result<()> {
    let env = Env::init()?;

    if env.is_git_hook("pre-commit") {
        return crate::hook::pre_commit(&env);
    }

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
        Some("test") => crate::test::do_test(&env),
        Some("doc") => crate::doc::do_doc(&env),
        Some("coverage") => crate::coverage::do_coverage(&env),
        Some("lint") => lint::do_lint(&env),
        Some("install-hooks") => crate::hook::install(&env),
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
