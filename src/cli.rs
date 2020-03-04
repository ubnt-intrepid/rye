#![allow(missing_docs)]

pub(crate) mod args;
pub(crate) mod exit_status;
pub(crate) mod session;

pub use self::{args::Args, exit_status::ExitStatus, session::Session};

pub fn install() {
    crate::global::install();
}
