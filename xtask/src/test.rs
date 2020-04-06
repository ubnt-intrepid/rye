use crate::env::Env;
use std::path::Path;

pub fn do_test(env: &Env) -> anyhow::Result<()> {
    crate::format::check(env)?;

    let testcrates_root = env.project_root().join("testcrates");

    run_crate_test(env, None)?;
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
                .current_dir(cwd)
                .run()?;
        }
    }

    Ok(())
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
        cargo().args(&["clippy", "--all-targets"]).run()?;
    }

    cargo().arg("test").run()?;

    Ok(())
}

fn is_nightly() -> bool {
    match version_check::Channel::read() {
        Some(ch) => ch.is_nightly(),
        _ => false,
    }
}
