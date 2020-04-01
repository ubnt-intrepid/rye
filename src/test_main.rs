#![allow(missing_docs)]

use crate::{
    global::install_globals, session::SessionInner, termination::Termination, test::TestCase,
};
use linkme::distributed_slice;

#[distributed_slice]
pub static TEST_CASES: [&'static dyn TestCase] = [..];

pub type TestCases<'a> = &'a [&'a dyn TestCase];

extern "Rust" {
    #[link_name = "__rye_test_main"]
    fn test_main(_: TestCases<'_>);
}

pub fn harness_main() {
    unsafe {
        test_main(&*TEST_CASES);
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
