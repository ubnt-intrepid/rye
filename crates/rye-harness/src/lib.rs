mod harness;

/// Re-exported items for test_harness!() and test_case!()
#[doc(hidden)]
pub mod _reexports {
    pub use {
        crate::harness::{main, TEST_CASES},
        linkme::{self, distributed_slice},
    };
}
