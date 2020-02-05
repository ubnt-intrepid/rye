/*!
Catch inspired testing framework for Rust.
!*/

// copied from pin-utils
#[doc(hidden)]
#[macro_export]
macro_rules! pin_mut {
    ($x:ident) => {
        let mut $x = $x;
        #[allow(unused_mut)]
        let mut $x = unsafe { std::pin::Pin::new_unchecked(&mut $x) };
    };
}

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
        let section = sections.root();
        pin_mut!(section);
        let _guard = crate::tls::set(section.as_mut());
        f();
    }
}

#[doc(hidden)]
pub mod _internal {
    use crate::section::Section;
    pub use crate::{section::SectionId, tls::set as set_section};

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
            $crate::pin_mut!(section);
            let _guard = $crate::_internal::set_section(section.as_mut());
            $body
        }
    }};
}
