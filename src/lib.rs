/*!
A Rust unit testing library inspired by Catch2.
!*/

mod section;
mod test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        section::{Section, SectionId},
        test_case::TestCase,
    };
}

/// Generate a test case.
pub use rye_macros::test_case;

/// Declare a section in the test case.
#[macro_export]
macro_rules! section {
    ($($t:tt)*) => {
        compile_error!("section!() cannot be used outside of #[test_case]")
    };
}
