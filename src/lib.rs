/*!
A Rust unit testing library inspired by Catch2.
!*/

mod context;
mod section;
mod test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::section::Section;
    pub use crate::test_case::TestCase;
    pub use phf::phf_set;

    use crate::{context::TestContext, section::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.section().is_target(id))
    }
}

/// Generate a test case.
pub use rye_macros::test_case;
