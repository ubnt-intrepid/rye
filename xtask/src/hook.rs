use crate::env::Env;
use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    fs,
    path::PathBuf,
};

fn resolve_git_dir(env: &Env) -> anyhow::Result<PathBuf> {
    let mut project_root = env.project_root().to_owned();
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

pub fn install(env: &Env) -> anyhow::Result<()> {
    let hooks_dir = resolve_git_dir(env)?.join("hooks");
    let me = current_exe()?;

    let install = |name: &str| -> anyhow::Result<()> {
        let hook_path = hooks_dir.join(format!("{}{}", name, EXE_SUFFIX));
        eprintln!(
            "[cargo-xtask] install {} to {}",
            me.display(),
            hook_path.display()
        );
        fs::copy(&me, hook_path)?;
        Ok(())
    };

    install("pre-commit")?;

    Ok(())
}

pub fn pre_commit(env: &Env) -> anyhow::Result<()> {
    crate::lint::do_lint(env)
}
