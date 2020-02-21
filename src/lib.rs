/*!
A custom unit testing framework inspired by Catch2.
!*/

pub mod executor;

mod registration;
mod test;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        registration::{Registration, Registry, RegistryError},
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

pub use crate::{
    registration::{Registration, Registry, RegistryError},
    test::Test,
};

/// Declare a single test.
pub use rye_macros::test;

/// Generate a main function.
pub use rye_macros::test_main;
