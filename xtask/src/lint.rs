use crate::shell::Shell;

pub fn do_lint(sh: &Shell) -> anyhow::Result<()> {
    let has_rustfmt = sh
        .cargo()
        .args(&["fmt", "--version"])
        .silent()
        .run()
        .is_ok();
    if has_rustfmt {
        sh.cargo().args(&["fmt", "--", "--check"]).run()?;
    }

    let has_clippy = sh
        .cargo()
        .args(&["clippy", "--version"])
        .silent()
        .run()
        .is_ok();

    let cargo_lint = || {
        if has_clippy {
            sh.cargo().args(&["clippy", "--all-targets"])
        } else {
            sh.cargo().args(&["check", "--all-targets"])
        }
    };

    cargo_lint().run()?;
    cargo_lint().arg("--package=smoke-harness").run()?;
    cargo_lint().arg("--package=smoke-frameworks").run()?;

    Ok(())
}
