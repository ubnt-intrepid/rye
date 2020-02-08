/*!
A Rust unit testing library inspired by Catch2.
!*/

mod futures;
mod section;
mod test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        section::{Section, SectionId},
        test_case::TestCase,
    };

    pub fn new_section(id: &'static SectionId) -> Option<Section> {
        Section::with(|section| section.new_section(id))
    }
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
