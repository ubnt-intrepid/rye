use crate::env::Env;
use std::path::Path;

pub fn do_lint(env: &Env) -> anyhow::Result<()> {
    let has_rustfmt = env
        .cargo()
        .args(&["fmt", "--version"])
        .silent()
        .run()
        .is_ok();

    let has_clippy = env
        .cargo()
        .args(&["clippy", "--version"])
        .silent()
        .run()
        .is_ok();

    let lint_crate = |cwd: Option<&Path>| -> anyhow::Result<()> {
        let mut target_dir = env.target_dir().to_path_buf();
        if let Some(cwd) = cwd {
            target_dir.push(cwd.file_stem().unwrap());
        }

        let cargo = || match cwd {
            Some(cwd) => env
                .cargo() //
                .current_dir(cwd)
                .env("CARGO_TARGET_DIR", &target_dir),
            None => env.cargo(),
        };

        if has_rustfmt {
            cargo().args(&["fmt", "--", "--check"]).run()?;
        }

        if has_clippy {
            cargo().args(&["clippy", "--all-targets"]).run()?;
        } else {
            cargo().args(&["check", "--all-targets"]).run()?;
        }

        Ok(())
    };

    lint_crate(None)?;
    lint_crate(Some(&env.project_root().join("testcrates/smoke-harness")))?;

    if env.is_nightly() {
        lint_crate(Some(
            &env.project_root().join("testcrates/smoke-frameworks"),
        ))?;
    }

    Ok(())
}
