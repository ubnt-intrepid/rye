use crate::env::Env;

pub fn do_test(env: &Env) -> anyhow::Result<()> {
    env.cargo().arg("test").run()?;

    env.cargo()
        .arg("test")
        .arg("--package=smoke-harness")
        .run()?;

    env.cargo()
        .arg("test")
        .arg("--package=smoke-frameworks")
        .run()?;

    if probe_cargo_wasi(env).is_ok() {
        env.cargo()
            .arg("wasi")
            .arg("test")
            .arg("--package=smoke-frameworks")
            .run()?;
    }

    Ok(())
}

fn probe_cargo_wasi(env: &Env) -> anyhow::Result<()> {
    env.subprocess("wasmtime").arg("--version").silent().run()?;
    env.cargo().args(&["wasi", "--version"]).silent().run()?;
    Ok(())
}
