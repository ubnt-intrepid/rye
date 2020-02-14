/*!
A custom unit testing framework inspired by Catch2.
!*/

mod runner;
mod test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        runner::{run_tests, TestSuite},
        test_case::{Section, TestCase, TestDesc, TestFn},
    };
    pub use maplit::{hashmap, hashset};
    pub use std::module_path;

    use crate::{runner::TestContext, test_case::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }
}

/// Generate a test case.
pub use rye_macros::test_case;

#[macro_export]
macro_rules! test_main {
    ($($test_case:path),*$(,)?) => {
        fn main() {
            $crate::_internal::run_tests(&[$(&$test_case),*]);
        }
    };
}
