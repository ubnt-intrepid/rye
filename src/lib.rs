/*!
Catch inspired testing framework for Rust.
!*/

#![cfg_attr(feature = "nightly", feature(doc_cfg))]

mod section;
mod test_case;
mod tls;

pub use crate::test_case::TestCase;

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
    ($name:expr, $body:block) => {{
        static SECTION: $crate::_internal::SectionId = $crate::_internal::SectionId::SubSection {
            name: $name,
            file: file!(),
            line: line!(),
            column: column!(),
        };
        if let Some(section) = $crate::_internal::new_section(&SECTION) {
            let _guard = $crate::_internal::Guard::set(Some(Box::new(section)));
            $body
        }
    }};
}
