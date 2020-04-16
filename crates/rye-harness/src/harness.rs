use linkme::distributed_slice;
use rye::{test_runner, TestCase};

#[doc(hidden)]
#[distributed_slice]
pub static TEST_CASES: [&'static TestCase] = [..];

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! test_case {
    ( $item:item ) => {
        #[$crate::_reexports::distributed_slice($crate::_reexports::TEST_CASES)]
        #[linkme(crate = $crate::_reexports::linkme)]
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
            $crate::_reexports::main()
        }
    };
}
