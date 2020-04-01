#![allow(missing_docs)]

use crate::{
    global::install_globals, session::SessionInner, termination::Termination, test::TestCase,
};

#[cfg(feature = "harness")]
#[linkme::distributed_slice]
pub static TEST_CASES: [&'static dyn TestCase] = [..];

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
    install_globals();

    let mut session = SessionInner::new(test_cases);
    let res = f(&mut session);

    crate::termination::exit(res)
}

/// Generate test harness.
#[cfg(feature = "harness")]
#[macro_export]
macro_rules! test_harness {
    () => {
        fn main() {
            $crate::_test_main_reexports::test_runner(&*$crate::_test_main_reexports::TEST_CASES);
        }
    };
}
