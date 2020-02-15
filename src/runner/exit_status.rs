/// Exit status code used as a result of the test process.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExitStatus(pub(crate) i32);

impl ExitStatus {
    pub(crate) const OK: Self = Self(0);
    pub(crate) const FAILED: Self = Self(101);

    /// Return whether the status is successful or not.
    #[inline]
    pub fn success(self) -> bool {
        self.code() == 0
    }

    /// Return the raw exit code.
    #[inline]
    pub fn code(self) -> i32 {
        self.0
    }

    /// Terminate the test process with the exit code.
    ///
    /// This method **should not** be called before the cleanup
    /// of the test process has completed.
    #[inline]
    pub fn exit(self) -> ! {
        std::process::exit(self.code());
    }

    /// Terminate the test process if the exit code is not successful.
    ///
    /// This method **should not** be called before the cleanup
    /// of the test process has completed.
    #[inline]
    pub fn exit_if_failed(self) {
        if !self.success() {
            self.exit();
        }
    }
}
