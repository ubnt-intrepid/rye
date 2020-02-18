/*!
A custom unit testing framework inspired by Catch2.
!*/

mod cli;
mod executor;
mod registry;
mod report;
mod runner;
mod test;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        cli::ExitStatus,
        executor::{DefaultTestExecutor, TestExecutor},
        registry::{Registration, Registry, RegistryError},
        runner::run_tests,
        test::{Section, Test, TestDesc, TestFn},
    };
    pub use futures::executor::block_on;
    pub use maplit::{hashmap, hashset};
    pub use std::{boxed::Box, module_path, result::Result, vec};

    use crate::{executor::TestContext, test::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }
}

/// Declare a single test.
pub use rye_macros::test;

/// Generate a main function.
pub use rye_macros::test_main;

#[doc(hidden)]
#[macro_export]
macro_rules! __annotate_test_case {
    ($item:item) => {
        $item
    };
}
