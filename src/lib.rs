/*!
A Rust unit testing library inspired by Catch2.
!*/

mod context;
mod desc;
mod runner;
mod section;

#[doc(hidden)]
pub mod _internal {
    pub use crate::desc::TestDesc;
    pub use crate::section::Section;
    pub use phf::phf_set;

    use crate::{context::TestContext, section::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.section().is_target(id))
    }
}

/// Generate a test case.
pub use rye_macros::test_case;

pub use crate::runner::{run_tests, DefaultRunner, TestData, TestSuite};

#[macro_export]
macro_rules! test_main {
    ($($test_case:path),*$(,)?) => {
        fn main() {
            $crate::run_tests($crate::DefaultRunner::new(), &[$(&$test_case),*]);
        }
    };
}
