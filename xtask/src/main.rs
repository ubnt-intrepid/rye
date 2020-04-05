use pico_args::Arguments;
use std::{
    env, fs,
    path::{Path, PathBuf},
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

fn run_crate_test(cwd: Option<&Path>) -> anyhow::Result<()> {
    eprintln!("[cargo-xtask] run_crate_test(cwd = {:?})", cwd);

    let cargo = || {
        let mut cargo = crate::cargo();
        if let Some(ref cwd) = cwd {
            cargo.current_dir(cwd);
        }
        cargo
    };

    if cargo().args(&["fmt", "--version"]).run_silent().is_ok() {
        cargo().args(&["fmt", "--", "--check"]).run()?;
    }

    if cargo().args(&["clippy", "--version"]).run_silent().is_ok() {
        cargo()
            .args(&["clippy", "--all-targets"])
            .env("RUSTFLAGS", "-D warnings")
            .run()?;
    }

    cargo() //
        .arg("test")
        .env("RUSTFLAGS", "-D warnings")
        .run()?;

    Ok(())
}

fn is_nightly() -> bool {
    match version_check::Channel::read() {
        Some(ch) => ch.is_nightly(),
        _ => false,
    }
}

fn do_test() -> anyhow::Result<()> {
    run_crate_test(None)?;

    let testcrates_root = project_root().join("testcrates");
    run_crate_test(Some(&testcrates_root.join("smoke-harness")))?;
    if is_nightly() {
        let cwd = testcrates_root.join("smoke-frameworks");
        run_crate_test(Some(&cwd))?;
        if cargo().arg("wasi").arg("--version").run_silent().is_ok() {
            cargo() //
                .arg("wasi")
                .arg("test")
                .env("RUSTFLAGS", "-D warnings")
                .current_dir(cwd)
                .run()?;
        }
    }

    Ok(())
}

fn do_docs() -> anyhow::Result<()> {
    let doc_dir = target_dir().join("doc");
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

    fs::remove_file(doc_dir.join(".lock"))?;

    Ok(())
}

fn do_coverage() -> anyhow::Result<()> {
    let target_dir = target_dir();

    let cov_dir = target_dir.join("cov");
    if cov_dir.exists() {
        fs::remove_dir_all(&cov_dir)?;
    }
    fs::create_dir_all(&cov_dir)?;

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

    if command("grcov").arg("--version").run_silent().is_ok() {
        command("grcov")
            .arg(env::current_dir().unwrap_or_else(|_| ".".into()))
            .arg("--branch")
            .arg("--ignore-not-existing")
            .arg("--llvm")
            .arg("--output-type")
            .arg("lcov")
            .arg("--output-file")
            .arg(cov_dir.join("lcov.info"))
            .run()?;
    }

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

fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    command.current_dir(project_root());
    command.stdin(Stdio::null());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    command
}

fn cargo() -> Command {
    let mut cargo = command(
        env::var_os("CARGO")
            .map(PathBuf::from)
            .unwrap_or_else(|| env!("CARGO").into()),
    );
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
        run_impl(self)
    }

    fn run_silent(&mut self) -> anyhow::Result<()> {
        self.stdout(Stdio::null()).stderr(Stdio::null());
        run_impl(self)
    }
}

fn run_impl(cmd: &mut Command) -> anyhow::Result<()> {
    if env::var_os("DRY_RUN").is_some() {
        eprintln!("[cargo-xtask] dry-run: {:#?}", cmd);
        return Ok(());
    }

    eprintln!("[cargo-xtask] run: {:#?}", cmd);
    let st = cmd.status()?;
    anyhow::ensure!(
        st.success(),
        "Subprocess failed with the exit code {}",
        st.code().unwrap_or(0),
    );
    Ok(())
}

fn project_root() -> PathBuf {
    let xtask_manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").to_owned().into());
    xtask_manifest_dir.ancestors().nth(1).unwrap().to_path_buf()
}

fn target_dir() -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root().join("target"))
}
