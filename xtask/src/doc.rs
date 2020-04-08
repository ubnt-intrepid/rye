use crate::shell::{RemoveFlags, Shell};
use std::net::SocketAddr;

pub fn do_doc(sh: &Shell, serve_addr: Option<SocketAddr>) -> anyhow::Result<()> {
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

    if probe_mdbook(sh).is_ok() {
        sh.subprocess("mdbook")
            .arg("build")
            .arg("-d")
            .arg(doc_dir.join("guide"))
            .arg(sh.project_root().join("docs"))
            .run()?;
    }

    sh.write(
        doc_dir.join("index.html"),
        "\
<meta http-equiv=\"refresh\" content=\"0; url=rye\">
<a href=\"rye\">Redirect</a>
",
    )?;

    sh.remove(doc_dir.join(".lock"), RemoveFlags::empty())?;

    if let Some(addr) = serve_addr {
        eprintln!("[cargo-xtask] start file server: {}", addr);

        sh.subprocess("python")
            .arg("-m")
            .arg("http.server")
            .arg(addr.port().to_string())
            .arg("--bind")
            .arg(addr.ip().to_string())
            .arg("--directory")
            .arg(&doc_dir)
            .run()?;
    }

    Ok(())
}

fn probe_mdbook(sh: &Shell) -> anyhow::Result<()> {
    sh.subprocess("mdbook").arg("--version").silent().run()
}
