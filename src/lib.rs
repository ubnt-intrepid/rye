/*!
A Rust unit testing library inspired by Catch2.
!*/

mod futures;
mod section;
mod test_case;
mod tls;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        section::{Section, SectionId},
        test_case::TestCase,
    };

    pub fn new_section(id: &'static SectionId) -> Option<Section> {
        crate::tls::with(|section| section.new_section(id))
    }

    #[inline]
    pub fn with_section<F, R>(section: &mut Section, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        crate::tls::set(section, f)
    }

    #[cfg(feature = "futures")]
    pub use crate::futures::with_section_async;
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
