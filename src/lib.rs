/*!
A Rust unit testing library inspired by Catch2.
!*/

mod futures;
mod section;
mod test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::section::Section;

    use crate::{section::SectionId, test_case::TestCase};

    pub fn new_section(id: u64, name: &'static str) -> Option<Section> {
        Section::with(|section| section.new_section(SectionId::new(id), name))
    }

    #[inline]
    pub fn run<F>(f: F)
    where
        F: Fn(),
    {
        let mut test_case = TestCase::new("root");
        test_case.scope(|| {
            while !TestCase::with(|test_case| test_case.completed()) {
                let mut section = Section::root();
                section.scope(&f);
            }
        })
    }

    #[cfg(feature = "futures")]
    pub use crate::futures::run_async;
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
