use crate::shell::{CreateFlags, RemoveFlags, Shell};

pub fn do_coverage(sh: &Shell) -> anyhow::Result<()> {
    let target_dir = sh.target_dir();

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
        sh.remove(&cov_dir, RemoveFlags::RECURSIVE)?;
    }
    sh.create_dir(&cov_dir, CreateFlags::RECURSIVE)?;

    sh.cargo()
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

    if sh
        .subprocess("grcov")
        .arg("--version")
        .silent()
        .run()
        .is_ok()
    {
        sh.subprocess("grcov")
            .arg(sh.project_root())
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
