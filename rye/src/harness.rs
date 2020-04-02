#![cfg(feature = "harness")]

use crate::{runner::test_runner, test::TestCase};
use linkme::distributed_slice;

#[doc(hidden)]
#[distributed_slice]
pub static TEST_CASES: [&'static dyn TestCase] = [..];

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __test_case_harness {
    ( $item:item ) => {
        #[$crate::_test_harness_reexports::distributed_slice(
            $crate::_test_harness_reexports::TEST_CASES
        )]
        #[linkme(crate = $crate::_test_harness_reexports::linkme)]
        $item
    };
}

#[doc(hidden)]
pub fn main() {
    test_runner(&*TEST_CASES);
}

/// Generate the main function for running test application.
#[macro_export]
macro_rules! test_harness {
    () => {
        fn main() {
            $crate::_test_harness_reexports::main()
        }
    };
}
