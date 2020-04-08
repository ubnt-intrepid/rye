/*!
A custom unit testing framework for Rust.

The concept is heavily influenced by the section mechanism in [`Catch2`](https://github.com/catchorg/Catch2),
a C++ unit testing framework library.
!*/

#![doc(html_root_url = "https://docs.rs/rye/0.1.0-dev")]
#![deny(missing_docs)]
#![forbid(clippy::unimplemented, clippy::todo)]

#[macro_use]
mod macros;
mod harness;
mod report;
mod runner;
mod session;
mod termination;
mod test;

pub use crate::{session::Session, termination::Termination, test::Context};

/// Generate a single test case.
pub use rye_macros::test;

/// Define a test main function.
pub use rye_macros::test_main;

#[doc(hidden)]
pub use runner::test_runner;

hidden_item! {
    /// Re-exported items for #[test]
    pub mod _test_reexports {
        pub use crate::{
            __location as location, //
            __section as section,
            __test_fn as test_fn,
            __test_name as test_name,
            termination::Termination,
            test::{
                Context, Location, Section, TestCase, TestDesc, TestFn, TestName, TestPlan,
            },
        };
        pub use std::{
            boxed::Box, column, concat, file, format_args, line, module_path, result::Result, stringify,
        };
    }

    /// Re-exported items for #[test_main]
    pub mod _test_main_reexports {
        pub use rye_runtime::{default_runtime, Runtime};
        pub use crate::{
            runner::{TestCases, test_main_inner},
        };
    }

    /// Re-exported items for test_harness!() and __test_case_harness!()
    #[cfg(feature = "harness")]
    pub mod _test_harness_reexports {
        pub use {
            crate::harness::{TEST_CASES, main},
            linkme::{self, distributed_slice},
        };
    }
}
