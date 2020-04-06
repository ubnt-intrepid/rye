use crate::env::Env;
use std::path::Path;

pub fn format(env: &Env) -> anyhow::Result<()> {
    format_impl(env, FormatMode::Overwrite)
}

pub fn check(env: &Env) -> anyhow::Result<()> {
    format_impl(env, FormatMode::CheckOnly)
}

#[derive(Copy, Clone, PartialEq)]
enum FormatMode {
    Overwrite,
    CheckOnly,
}

fn format_impl(env: &Env, mode: FormatMode) -> anyhow::Result<()> {
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
