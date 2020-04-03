use pico_args::Arguments;
use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
    env::set_current_dir(project_root())?;

    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();
    match &*subcommand {
        _ => {
            eprintln!(
                "\
cargo-xtask
Free style automation tool

Usage:
    cargo xtask <SUBCOMMAND>

Subcommands:
"
            );
            Ok(())
        }
    }
}

fn project_root() -> PathBuf {
    let xtask_manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").to_owned().into());
    xtask_manifest_dir.ancestors().nth(1).unwrap().to_path_buf()
}
