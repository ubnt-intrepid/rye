/*!
Catch inspired testing framework for Rust.
!*/

mod section;
mod tls;

use crate::{section::Sections, tls::Guard};

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        mod futures;
        pub use futures::test_case_async;
    }
}

/// Run a test case.
pub fn test_case<'a, F>(f: F)
where
    F: Fn() + 'a,
{
    let sections = Sections::new();
    while !sections.completed() {
        let section = sections.root();
        let _guard = Guard::set(Some(Box::new(section)));
        f();
    }
}

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
