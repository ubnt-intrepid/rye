/*!
A custom unit testing framework inspired by Catch2.
!*/

mod cli;
mod executor;
mod registry;
mod report;
mod runner;
mod session;
mod test;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        cli::ExitStatus,
        registry::{Registration, Registry, RegistryError},
        runner::default_runner,
        test::{Section, Test, TestDesc, TestFn},
    };
    pub use maplit::{hashmap, hashset};
    pub use std::{boxed::Box, module_path, result::Result, vec};

    use crate::{executor::TestContext, test::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __annotate_test_case {
        ($item:item) => {
            $item
        };
    }
}

/// Declare a single test.
pub use rye_macros::test;

/// Generate a main function.
pub use rye_macros::test_main;
