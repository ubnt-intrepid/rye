use bitflags::bitflags;
use fakeenv::EnvStore;
use std::{
    ffi::OsStr,
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
};

/// The minimal implementation of shell for xtask scripts.
pub struct Shell {
    env_store: EnvStore,
    project_root: PathBuf,
    target_dir: PathBuf,
    _anchor: PhantomData<Rc<()>>, // FIXME: make thread safe
}

impl Shell {
    pub fn new() -> Self {
        let envs = EnvStore::fake();

        let manifest_dir = envs
            .var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .or_else(|| option_env!("CARGO_MANIFEST_DIR").map(PathBuf::from))
            .expect("missing CARGO_MANIFEST_DIR");
        let project_root = manifest_dir.ancestors().nth(1).unwrap().to_path_buf();

        let target_dir = envs
            .var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| project_root.join("target"));

        Self {
            env_store: envs,
            project_root,
            target_dir,
            _anchor: PhantomData,
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    // ```
    // $ mkdir {{ path }} {{ flags }}
    // ```
    pub fn create_dir(&self, path: impl AsRef<Path>, flags: CreateFlags) -> anyhow::Result<()> {
        if flags.contains(CreateFlags::RECURSIVE) {
            fs::create_dir_all(path)?;
        } else {
            fs::create_dir(path)?;
        }
        Ok(())
    }

    // ```
    // $ cat << EOF > {{ path }}
    // {{ content }}
    // EOF
    // ```
    pub fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> anyhow::Result<()> {
        fs::write(path, content)?;
        Ok(())
    }

    // ```
    // $ rm {{ path }} {{ flags }}
    // ```
    pub fn remove(&self, path: impl AsRef<Path>, flags: RemoveFlags) -> anyhow::Result<()> {
        let path = path.as_ref();

        if path.is_dir() {
            if flags.contains(RemoveFlags::RECURSIVE) {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_dir(path)?;
            }

            return Ok(());
        }

        if path.is_file() {
            fs::remove_file(path)?;

            return Ok(());
        }

        Ok(())
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

    pub fn rustc(&self) -> Subprocess {
        self.subprocess(
            self.env_store
                .var_os("RUSTC")
                .or_else(|| option_env!("RUSTC").map(Into::into))
                .unwrap_or_else(|| "rustc".into()),
        )
    }

    pub fn cargo(&self) -> Subprocess {
        self.subprocess(
            self.env_store
                .var_os("CARGO")
                .or_else(|| option_env!("CARGO").map(Into::into))
                .unwrap_or_else(|| "cargo".into()),
        )
    }
}

bitflags! {
    pub struct CreateFlags: u32 {
        const RECURSIVE = 0b_0000_0001;
    }
}

bitflags! {
    pub struct RemoveFlags: u32 {
        const RECURSIVE = 0b_0000_0001;
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
