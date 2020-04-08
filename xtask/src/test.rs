use crate::shell::Shell;

pub fn do_test(sh: &Shell) -> anyhow::Result<()> {
    sh.cargo().arg("test").run()?;

    sh.cargo()
        .arg("test")
        .arg("--package=smoke-harness")
        .run()?;

    sh.cargo()
        .arg("test")
        .arg("--package=smoke-frameworks")
        .run()?;

    if probe_cargo_wasi(sh).is_ok() {
        sh.cargo()
            .arg("wasi")
            .arg("test")
            .arg("--package=smoke-frameworks")
            .run()?;
    }

    Ok(())
}

fn probe_cargo_wasi(sh: &Shell) -> anyhow::Result<()> {
    sh.subprocess("wasmtime").arg("--version").silent().run()?;
    sh.cargo().args(&["wasi", "--version"]).silent().run()?;
    Ok(())
}
