use crate::env::Env;
use std::path::Path;

pub fn do_test(env: &Env) -> anyhow::Result<()> {
    let run_crate_test = |cwd: Option<&Path>| -> anyhow::Result<()> {
        let mut target_dir = env.target_dir().to_path_buf();
        if let Some(cwd) = cwd {
            target_dir.push(cwd.file_stem().unwrap());
        }

        let cargo = || {
            env.cargo() //
                .env("CARGO_TARGET_DIR", &target_dir)
                .if_some(cwd, |cargo, cwd| cargo.current_dir(cwd))
        };

        cargo().arg("test").run()?;

        Ok(())
    };

    run_crate_test(None)?;
    run_crate_test(Some(&env.project_root().join("testcrates/smoke-harness")))?;

    if env.is_nightly() {
        let cwd = env.project_root().join("testcrates/smoke-frameworks");
        run_crate_test(Some(&cwd))?;

        if env
            .cargo()
            .args(&["wasi", "--version"])
            .silent()
            .run()
            .is_ok()
        {
            env.cargo()
                .args(&["wasi", "test"])
                .env(
                    "CARGO_TARGET_DIR",
                    env.target_dir().join("smoke-frameworks"),
                )
                .current_dir(cwd)
                .run()?;
        }
    }

    Ok(())
}
