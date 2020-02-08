/*!
A Rust unit testing library inspired by Catch2.
!*/

mod section;

#[doc(hidden)]
pub mod _internal {
    pub use crate::section::Section;
    pub use phf::phf_set;

    #[inline]
    pub fn run<'a, F>(f: F, sections: &'a [Section])
    where
        F: Fn(&'a Section),
    {
        if sections.is_empty() {
            f(&Section::ROOT);
            return;
        }

        for section in sections {
            if section.is_leaf() {
                f(section);
            }
        }
    }

    #[cfg(feature = "futures")]
    #[inline]
    pub async fn run_async<'a, F, Fut>(f: F, sections: &'a [Section])
    where
        F: Fn(&'a Section) -> Fut,
        Fut: futures_core::Future + 'a,
    {
        if sections.is_empty() {
            f(&Section::ROOT).await;
            return;
        }

        for section in sections {
            if section.is_leaf() {
                f(section).await;
            }
        }
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
