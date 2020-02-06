/*!
Catch inspired testing framework for Rust.
!*/

#![cfg_attr(feature = "nightly", feature(doc_cfg))]

mod section;
mod test_case;
mod tls;

pub use crate::test_case::TestCase;
pub use rye_macros::test_case;

#[doc(hidden)]
pub mod _internal {
    use crate::section::Section;
    pub use crate::{section::SectionId, tls::Guard};

    #[inline]
    pub fn new_section(id: &'static SectionId) -> Option<Section> {
        crate::tls::with(|section| section.new_section(id))
    }
}

/// Declare a section in the test case.
#[macro_export]
macro_rules! section {
    ($($t:tt)*) => {
        compile_error!("section!() cannot be used outside of #[test_case]")
    };
}
