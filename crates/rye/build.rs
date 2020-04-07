use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitStatus},
};

fn main() {
    if cfg!(feature = "harness") {
        println!("cargo:rustc-cfg=harness");
    }

    if cfg!(feature = "frameworks")
        && !cfg!(feature = "harness")
        && probe_custom_test_frameworks().map_or(false, |st| st.success())
    {
        println!("cargo:rustc-cfg=frameworks");
    }
}

fn probe_custom_test_frameworks() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR").map(PathBuf::from)?;
    let probefile = out_dir.join("probe_ctf.rs");

    fs::write(
        &probefile,
        r#"
            #![feature(custom_test_frameworks)]
            #![test_runner(crate::runner)]

            #[test_case]
            const A: u32 = 42;

            fn runner(_: &[&u32]) {}
        "#,
    )
    .ok()?;

    Command::new(rustc)
        .arg("--edition=2018")
        .arg("--crate-name=rye_build_probe_ctf")
        .arg("--test")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile)
        .status()
        .ok()
}
