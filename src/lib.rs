/*!
Catch inspired testing framework for Rust.
!*/

#![cfg_attr(feature = "nightly", feature(doc_cfg))]

mod section;
mod test_case;

pub use rye_macros::test_case;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        section::{Section, SectionId},
        test_case::TestCase,
    };
}

/// Declare a section in the test case.
#[macro_export]
macro_rules! section {
    ($($t:tt)*) => {
        compile_error!("section!() cannot be used outside of #[test_case]")
    };
}
