#![allow(missing_docs)]

use crate::{session::SessionInner, termination::Termination, test::TestCase};
use maybe_unwind::capture_panic_info;
use std::panic;
use std::sync::Once;

pub type TestCases<'a> = &'a [&'a dyn TestCase];

extern "Rust" {
    #[link_name = "__rye_test_main"]
    fn __rye_test_main(_: TestCases<'_>);
}

pub fn test_runner(test_cases: TestCases<'_>) {
    unsafe {
        __rye_test_main(test_cases);
    }
}

pub fn test_main_inner<F, R>(test_cases: TestCases, f: F)
where
    F: FnOnce(&mut SessionInner) -> R,
    R: Termination,
{
    static INSTALL_GLOBALS: Once = Once::new();
    INSTALL_GLOBALS.call_once(|| {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !capture_panic_info(info) {
                prev_hook(info);
            }
        }));
    });

    let mut session = SessionInner::new(test_cases);
    let res = f(&mut session);

    let code = match Termination::into_result(res) {
        Ok(()) => 0,
        Err(_) => 101,
    };
    std::process::exit(code);
}

#[cfg(test)]
#[export_name = "__rye_test_main"]
fn dummy(_: TestCases<'_>) {}
