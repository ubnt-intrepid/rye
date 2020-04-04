use pico_args::Arguments;
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

fn main() -> anyhow::Result<()> {
    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();
    match &*subcommand {
        "ci" => {
            let subcommand = args.subcommand()?.unwrap_or_default();
            match &*subcommand {
                "test" => do_test(),
                "docs" => do_docs(),
                "coverage" => do_coverage(),
                _ => {
                    eprintln!(
                        "\
cargo-xtask ci
Run CI scripts

Usage:
    cargo xtask ci <SUBCOMMAND> [FLAGS]

Subcommands:
        test        run CI flow
        coverage    run coverage test
        docs        generate API docs
"
                    );
                    Ok(())
                }
            }
        }
        "pre-commit" => run_pre_commit_hook(),
        _ => {
            eprintln!(
                "\
cargo-xtask
Free style automation tool

Usage:
    cargo xtask <SUBCOMMAND>

Subcommands:
        ci          Run CI scripts
        pre-commit  Run Git pre-commit hook
"
            );
            anyhow::bail!("invalid CLI argument")
        }
    }
}

fn do_test() -> anyhow::Result<()> {
    if cargo().args(&["fmt", "--version"]).run_silent().is_ok() {
        cargo() //
            .arg("fmt")
            .args(&["--all", "--", "--check"])
            .run()?;
    }

    if cargo().args(&["clippy", "--version"]).run_silent().is_ok() {
        cargo()
            .arg("clippy")
            .arg("--all-targets")
            .env("RUSTFLAGS", "-D warnings")
            .run()?;
    }

    cargo() //
        .arg("test")
        .env("RUSTFLAGS", "-D warnings")
        .run()?;

    Ok(())
}

fn do_docs() -> anyhow::Result<()> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root().join("target"));

    let doc_dir = target_dir.join("doc");
    if doc_dir.exists() {
        fs::remove_dir_all(&doc_dir)?;
    }

    fn cargo_rustdoc(package: &str) -> Command {
        let mut cargo = cargo();
        cargo
            .arg("rustdoc")
            .arg("--package")
            .arg(package)
            .arg("--")
            .args(&["--cfg", "docs"]);
        cargo
    }

    cargo_rustdoc("rye").run()?;
    cargo_rustdoc("rye-runtime").run()?;
    cargo_rustdoc("rye-runtime-tokio").run()?;

    fs::remove_file(target_dir.join("doc").join(".lock"))?;

    Ok(())
}

fn do_coverage() -> anyhow::Result<()> {
    if let Some((_version, channel, date)) = version_check::triple() {
        anyhow::ensure!(
            channel.is_nightly(),
            "coverage test is available only on nightly channel"
        );
        anyhow::ensure!(
            date.at_most("2020-03-14"),
            "coverage test was broken since 2020-03-15"
        );
    }

    cargo()
        .arg("test")
        .env(
            "RUSTFLAGS",
            "\
                -Zprofile \
                -Ccodegen-units=1 \
                -Cinline-threshold=0 \
                -Clink-dead-code \
                -Coverflow-checks=off \
                -Zno-landing-pads\
            ",
        )
        .run()?;

    Ok(())
}

fn run_pre_commit_hook() -> anyhow::Result<()> {
    cargo()
        .arg("fmt")
        .arg("--all")
        .arg("--")
        .arg("--check")
        .run()?;
    Ok(())
}

fn cargo() -> Command {
    let project_root = project_root();
    let cargo_env = env::var_os("CARGO")
        .map(PathBuf::from)
        .unwrap_or_else(|| env!("CARGO").into());

    let mut cargo = Command::new(cargo_env);
    cargo.stdin(Stdio::null());
    cargo.stdout(Stdio::inherit());
    cargo.stderr(Stdio::inherit());
    cargo.current_dir(&project_root);
    cargo.env("CARGO_INCREMENTAL", "0");
    cargo.env("CARGO_NET_OFFLINE", "true");
    cargo.env("RUST_BACKTRACE", "full");

    cargo
}

trait CommandExt {
    fn run(&mut self) -> anyhow::Result<()>;
    fn run_silent(&mut self) -> anyhow::Result<()>;
}

impl CommandExt for Command {
    fn run(&mut self) -> anyhow::Result<()> {
        eprintln!("[cargo-xtask] {:#?}", self);
        let st = self.status()?;
        anyhow::ensure!(
            st.success(),
            "Subprocess failed with the exit code {}",
            st.code().unwrap_or(0),
        );
        Ok(())
    }

    fn run_silent(&mut self) -> anyhow::Result<()> {
        let st = self.stdout(Stdio::null()).stderr(Stdio::null()).status()?;
        anyhow::ensure!(
            st.success(),
            "Subprocess failed with the exit code {}",
            st.code().unwrap_or(0),
        );
        Ok(())
    }
}

fn project_root() -> PathBuf {
    let xtask_manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").to_owned().into());
    xtask_manifest_dir.ancestors().nth(1).unwrap().to_path_buf()
}
