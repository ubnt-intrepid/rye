use pico_args::Arguments;
use std::{fs, path::Path};
use xtask::{env::Env, process::Subprocess};

fn main() -> anyhow::Result<()> {
    let env = Env::init()?;

    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();
    match &*subcommand {
        "ci" => {
            let subcommand = args.subcommand()?.unwrap_or_default();
            match &*subcommand {
                "test" => do_test(&env),
                "docs" => do_docs(&env),
                "coverage" => do_coverage(&env),
                _ => {
                    eprintln!(
                        "\
cargo-xtask ci
Run CI scripts

Usage:
    cargo xtask ci <SUBCOMMAND> [FLAGS]

Subcommands:
        test        run test
        coverage    run coverage test
        docs        generate API docs
"
                    );
                    Ok(())
                }
            }
        }
        "fmt" => do_format(&env, FormatMode::Overwrite),
        "pre-commit" => do_pre_commit_hook(&env),
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

fn run_crate_test(env: &Env, cwd: Option<&Path>) -> anyhow::Result<()> {
    eprintln!("[cargo-xtask] run_crate_test");

    let mut target_dir = env.target_dir().join("ci");

    if let Some(cwd) = cwd {
        eprintln!(
            "[cargo-xtask] - specified package location: {})",
            cwd.display()
        );
        target_dir.push(cwd.file_stem().unwrap());
    }

    let cargo = || {
        env.cargo() //
            .env("CARGO_TARGET_DIR", &target_dir)
            .if_some(cwd, |cargo, cwd| cargo.current_dir(cwd))
    };

    if cargo()
        .args(&["clippy", "--version"])
        .silent()
        .run()
        .is_ok()
    {
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

fn do_test(env: &Env) -> anyhow::Result<()> {
    run_crate_test(env, None)?;

    let testcrates_root = env.project_root().join("testcrates");
    run_crate_test(env, Some(&testcrates_root.join("smoke-harness")))?;
    if is_nightly() {
        let cwd = testcrates_root.join("smoke-frameworks");
        run_crate_test(env, Some(&cwd))?;
        if env
            .cargo()
            .args(&["wasi", "--version"])
            .silent()
            .run()
            .is_ok()
        {
            env.cargo()
                .args(&["wasi", "test"])
                .env("CARGO_TARGET_DIR", env.target_dir().join("ci"))
                .env("RUSTFLAGS", "-D warnings")
                .current_dir(cwd)
                .run()?;
        }
    }

    Ok(())
}

fn do_docs(env: &Env) -> anyhow::Result<()> {
    let doc_dir = env.target_dir().join("doc");
    if doc_dir.exists() {
        fs::remove_dir_all(&doc_dir)?;
    }

    fn cargo_rustdoc(env: &Env, package: &str) -> Subprocess {
        env.cargo()
            .arg("rustdoc")
            .arg("--package")
            .arg(package)
            .args(&["--", "--cfg", "docs"])
    }

    cargo_rustdoc(env, "rye").run()?;
    cargo_rustdoc(env, "rye-runtime").run()?;
    cargo_rustdoc(env, "rye-runtime-tokio").run()?;

    fs::remove_file(doc_dir.join(".lock"))?;

    Ok(())
}

fn do_coverage(env: &Env) -> anyhow::Result<()> {
    let target_dir = env.target_dir();

    if let Some((_version, channel, date)) = version_check::triple() {
        if !channel.is_nightly() {
            eprintln!("[cargo-xtask] coverage test is available only on nightly channel");
            return Ok(());
        }

        if !date.at_most("2020-03-14") {
            eprintln!("[cargo-xtask] warning: coverage test was broken since 2020-03-15");
        }
    }

    let cov_dir = target_dir.join("cov");
    if cov_dir.exists() {
        fs::remove_dir_all(&cov_dir)?;
    }
    fs::create_dir_all(&cov_dir)?;

    env.cargo()
        .arg("test")
        .arg("--target-dir")
        .arg(&cov_dir)
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

    if env
        .subprocess("grcov")
        .arg("--version")
        .silent()
        .run()
        .is_ok()
    {
        env.subprocess("grcov")
            .arg(env.project_root())
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

fn do_pre_commit_hook(env: &Env) -> anyhow::Result<()> {
    do_format(env, FormatMode::CheckOnly)?;
    Ok(())
}

#[derive(Copy, Clone, PartialEq)]
enum FormatMode {
    Overwrite,
    CheckOnly,
}

fn do_format(env: &Env, mode: FormatMode) -> anyhow::Result<()> {
    if env
        .cargo()
        .args(&["fmt", "--version"])
        .silent()
        .run()
        .is_err()
    {
        eprintln!("[cargo-xtask] cargo-fmt is not installed");
        return Ok(());
    }

    let cargo_fmt_check = |cwd: Option<&Path>| {
        env.cargo()
            .arg("fmt")
            .if_true(mode == FormatMode::CheckOnly, |cargo| {
                cargo.args(&["--", "--check"])
            })
            .if_some(cwd, |cargo, cwd| cargo.current_dir(cwd))
            .run()
    };

    let testcrates = env.project_root().join("testcrates");
    cargo_fmt_check(None)?;
    cargo_fmt_check(Some(&testcrates.join("smoke-harness")))?;
    cargo_fmt_check(Some(&testcrates.join("smoke-frameworks")))?;

    Ok(())
}
