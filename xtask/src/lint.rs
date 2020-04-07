use crate::env::Env;

pub fn do_lint(env: &Env) -> anyhow::Result<()> {
    let has_rustfmt = env
        .cargo()
        .args(&["fmt", "--version"])
        .silent()
        .run()
        .is_ok();
    if has_rustfmt {
        env.cargo().args(&["fmt", "--", "--check"]).run()?;
    }

    let has_clippy = env
        .cargo()
        .args(&["clippy", "--version"])
        .silent()
        .run()
        .is_ok();

    let cargo_lint = || {
        if has_clippy {
            env.cargo().args(&["clippy", "--all-targets"])
        } else {
            env.cargo().args(&["check", "--all-targets"])
        }
    };

    cargo_lint().run()?;
    cargo_lint().arg("--package=smoke-harness").run()?;
    cargo_lint().arg("--package=smoke-frameworks").run()?;

    Ok(())
}
