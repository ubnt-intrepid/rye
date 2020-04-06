use crate::env::Env;
use std::fs;

pub fn do_doc(env: &Env) -> anyhow::Result<()> {
    let doc_dir = env.target_dir().join("doc");
    if doc_dir.exists() {
        fs::remove_dir_all(&doc_dir)?;
    }

    let cargo_rustdoc = |package: &str| {
        env.cargo()
            .arg("rustdoc")
            .arg("--package")
            .arg(package)
            .args(&["--", "--cfg", "docs"])
    };

    cargo_rustdoc("rye").run()?;
    cargo_rustdoc("rye-runtime").run()?;
    cargo_rustdoc("rye-runtime-tokio").run()?;

    fs::remove_file(doc_dir.join(".lock"))?;

    Ok(())
}
