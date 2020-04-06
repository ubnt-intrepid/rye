use fakeenv::EnvStore;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// The collection of environment information.
#[derive(Debug)]
pub struct Env {
    env_store: EnvStore,
    project_root: PathBuf,
    target_dir: PathBuf,
}

impl Env {
    pub fn init() -> anyhow::Result<Self> {
        let env_store = EnvStore::real().to_fake();

        let xtask_manifest_dir = env_store
            .var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .or_else(|| option_env!("CARGO_MANIFEST_DIR").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("./xtask"));

        let project_root = xtask_manifest_dir.ancestors().nth(1).unwrap().to_path_buf();

        let target_dir = env_store
            .var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| project_root.join("target"));

        Ok(Self {
            env_store,
            project_root,
            target_dir,
        })
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    pub fn is_git_hook(&self, name: &str) -> bool {
        std::env::args().next().map_or(false, |s| s.contains(name))
    }

    pub fn is_nightly(&self) -> bool {
        match version_check::Channel::read() {
            Some(ch) => ch.is_nightly(),
            _ => false,
        }
    }

    pub fn subprocess(&self, program: impl AsRef<OsStr>) -> Subprocess {
        let dry_run = self.env_store.var_os("DRY_RUN").is_some();

        let mut command = Command::new(program);
        command.current_dir(&self.project_root);
        command.env_clear();
        command.envs(self.env_store.vars_os());

        command.stdin(Stdio::null());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        Subprocess { command, dry_run }
    }

    pub fn cargo(&self) -> Subprocess {
        self.subprocess(
            self.env_store
                .var_os("CARGO")
                .or_else(|| option_env!("CARGO").map(Into::into))
                .unwrap_or_else(|| "cargo".into()),
        )
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_NET_OFFLINE", "true")
        .env("RUST_BACKTRACE", "full")
    }
}

/// A thin wrapper to improve the convenience of `std::process::Command`.
pub struct Subprocess {
    command: Command,
    dry_run: bool,
}

impl Subprocess {
    pub fn arg<S>(mut self, arg: S) -> Self
    where
        S: AsRef<OsStr>,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }

    pub fn current_dir<P>(mut self, dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.command.current_dir(dir);
        self
    }

    pub fn if_some<T>(self, val: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        if let Some(val) = val {
            f(self, val)
        } else {
            self
        }
    }

    pub fn silent(mut self) -> Self {
        self.command.stdout(Stdio::null());
        self.command.stderr(Stdio::null());
        self
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        eprintln!("[cargo-xtask] run: {:#?}", self.command);

        if self.dry_run {
            eprintln!("[cargo-xtask] - skipped");
            return Ok(());
        }

        let st = self.command.status()?;
        anyhow::ensure!(
            st.success(),
            "Subprocess failed with the exit code {}",
            st.code().unwrap_or(0),
        );

        Ok(())
    }
}
