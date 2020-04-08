use crate::shell::{RemoveFlags, Shell};

pub fn do_doc(sh: &Shell) -> anyhow::Result<()> {
    let doc_dir = sh.target_dir().join("doc");
    if doc_dir.exists() {
        sh.remove(&doc_dir, RemoveFlags::RECURSIVE)?;
    }

    let cargo_rustdoc = |package: &str| {
        sh.cargo()
            .arg("rustdoc")
            .arg("--package")
            .arg(package)
            .args(&["--", "--cfg", "docs"])
    };

    cargo_rustdoc("rye").run()?;
    cargo_rustdoc("rye-runtime").run()?;
    cargo_rustdoc("rye-runtime-tokio").run()?;

    sh.remove(doc_dir.join(".lock"), RemoveFlags::empty())?;

    Ok(())
}
