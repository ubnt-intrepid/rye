use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

/// A thin wrapper to improve the convenience of `std::process::Command`.
pub struct Subprocess {
    pub(crate) command: Command,
    pub(crate) dry_run: bool,
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
