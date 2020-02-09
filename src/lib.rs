/*!
A Rust unit testing library inspired by Catch2.
!*/

mod context;
mod section;

#[doc(hidden)]
pub mod _internal {
    pub use crate::section::Section;
    pub use phf::phf_set;

    use crate::{context::TestContext, section::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.section().is_target(id))
    }

    #[inline]
    pub fn run<F>(f: F, sections: &[Section])
    where
        F: Fn(),
    {
        if sections.is_empty() {
            TestContext::new(&Section::ROOT).scope(&f);
            return;
        }

        for section in sections {
            if section.is_leaf() {
                TestContext::new(section).scope(&f);
            }
        }
    }

    #[cfg(feature = "futures")]
    #[inline]
    pub async fn run_async<F, Fut>(f: F, sections: &[Section])
    where
        F: Fn() -> Fut,
        Fut: futures_core::Future<Output = ()>,
    {
        if sections.is_empty() {
            TestContext::new(&Section::ROOT).scope_async(f()).await;
            return;
        }

        for section in sections {
            if section.is_leaf() {
                TestContext::new(section).scope_async(f()).await;
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
