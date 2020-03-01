#![allow(missing_docs)]

mod args;
mod exit_status;
mod report;
mod session;

pub use self::{exit_status::ExitStatus, session::Session};

pub fn install() {
    crate::global::install();
}
