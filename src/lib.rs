/*!
Catch inspired testing framework for Rust.
!*/

mod section;
mod tls;

use crate::section::Sections;

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
        let mut section = sections.root();
        let _guard = crate::tls::set(&mut section);
        f();
    }
}

#[doc(hidden)]
pub mod _internal {
    use crate::section::Section;
    pub use crate::{section::SectionId, tls::set as set_session};

    #[inline]
    pub fn new_session(id: &'static SectionId) -> Option<Section> {
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
        if let Some(mut section) = $crate::_internal::new_session(&SECTION) {
            let _guard = $crate::_internal::set_session(&mut section);
            $body
        }
    }};
}
