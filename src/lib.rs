/*!
A custom unit testing framework inspired by Catch2.
!*/

mod cli;
mod executor;
mod registry;
mod report;
mod runner;
mod test;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        cli::ExitStatus,
        executor::{DefaultTestExecutor, TestExecutor},
        registry::{Registry, RegistryError},
        runner::run_tests,
        test::{Section, Test, TestDesc, TestFn},
    };
    pub use futures::executor::block_on;
    pub use maplit::{hashmap, hashset};
    pub use std::module_path;

    use crate::{executor::TestContext, test::SectionId};
    use std::borrow::Cow;

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }

    #[inline]
    pub fn test_name(module_path: &'static str, title: &'static str) -> Cow<'static, str> {
        match module_path.splitn(2, "::").nth(1) {
            Some("") | None => Cow::Borrowed(title),
            Some(module) => Cow::Owned(format!("{}::{}", module, title)),
        }
    }

    #[macro_export]
    macro_rules! test_main {
        ($($test_case:path),*$(,)?) => {
            fn main() {
                let mut executor = $crate::_internal::DefaultTestExecutor::new().unwrap();
                let status = $crate::_internal::block_on(
                    $crate::_internal::run_tests(&[$(&$test_case),*], &mut executor)
                );
                status.exit();
            }
        };
    }
}

/// Declare a single test.
pub use rye_macros::test;
