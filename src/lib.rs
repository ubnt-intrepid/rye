/*!
A custom unit testing framework inspired by Catch2.
!*/

mod args;
mod context;
mod executor;
mod exit_status;
mod outcome;
mod printer;
mod report;
mod runner;
mod test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        executor::{DefaultTestExecutor, TestExecutor},
        exit_status::ExitStatus,
        runner::{run_tests, TestSuite},
        test_case::{Section, TestCase, TestDesc, TestFn},
    };
    pub use futures::executor::block_on;
    pub use maplit::{hashmap, hashset};
    pub use std::module_path;

    use crate::{context::TestContext, test_case::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }

    #[macro_export]
    macro_rules! test_main {
        ($($test_case:path),*$(,)?) => {
            fn main() {
                let mut executor = $crate::_internal::DefaultTestExecutor::new().unwrap();
                let status = $crate::_internal::block_on(
                    $crate::_internal::run_tests(&[$(&$test_case),*], &mut executor)
                );
                status.exit();
            }
        };
    }
}

/// Generate a test case.
pub use rye_macros::test_case;
