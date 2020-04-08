use crate::shell::{CreateFlags, Shell};
use std::path::PathBuf;

fn resolve_git_dir(sh: &Shell) -> anyhow::Result<PathBuf> {
    let mut project_root = sh.project_root().to_owned();
    if !project_root.has_root() {
        project_root = project_root.canonicalize()?;
    }

    for dir in project_root.ancestors() {
        let git_dir = dir.join(".git");
        if git_dir.is_dir() {
            return Ok(git_dir);
        }
    }

    anyhow::bail!("Git directory is not found");
}

pub fn install(sh: &Shell) -> anyhow::Result<()> {
    let hooks_dir = resolve_git_dir(sh)?.join("hooks");

    let install = |name: &str| -> anyhow::Result<()> {
        eprintln!(
            "[cargo-xtask] install hook {} to {}",
            name,
            hooks_dir.display()
        );

        let hook_src_dir = sh.target_dir().join("xtask");
        sh.create_dir(&hook_src_dir, CreateFlags::RECURSIVE)?;

        let hook_src = hook_src_dir.join(format!("{}.rs", name));
        sh.write(
            &hook_src,
            format!(
                r#"
                    fn main() -> std::io::Result<()> {{
                        let status = std::process::Command::new("cargo")
                            .arg("xtask")
                            .arg("{name}")
                            .status()?;
                        std::process::exit(status.code().unwrap_or(0));
                    }}
                "#,
                name = name
            ),
        )?;

        sh.rustc()
            .arg("--edition=2018")
            .arg("--crate-type=bin")
            .arg("--out-dir")
            .arg(&hooks_dir)
            .arg(&hook_src)
            .run()?;

        Ok(())
    };

    install("pre-commit")?;

    Ok(())
}

pub fn pre_commit(env: &Shell) -> anyhow::Result<()> {
    println!("[cargo-xtask] run pre-commit hook");
    crate::lint::do_lint(env)
}
