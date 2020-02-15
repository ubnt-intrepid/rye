/*!
A custom unit testing framework inspired by Catch2.
!*/

mod args;
mod context;
mod driver;
mod exit_status;
mod outcome;
mod printer;
mod report;
mod test_case;
mod test_suite;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        test_case::{Section, TestCase, TestDesc, TestFn},
        test_suite::TestSuite,
    };
    pub use futures::executor::block_on;
    pub use maplit::{hashmap, hashset};
    pub use std::module_path;

    use crate::{args::Args, context::TestContext, driver::TestDriver, test_case::SectionId};
    use std::sync::Once;

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }

    #[inline(never)]
    pub async fn run_tests(tests: &[&dyn Fn(&mut TestSuite<'_>)]) {
        let args = Args::from_env().unwrap_or_else(|st| st.exit());

        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            maybe_unwind::set_hook();
        });

        let mut test_cases = vec![];
        for &test in tests {
            test(&mut TestSuite {
                test_cases: &mut test_cases,
            });
        }

        let mut driver = TestDriver::new(&args);
        let st = match driver.run_tests(test_cases).await {
            Ok(report) => report.status(),
            Err(status) => status,
        };
        st.exit();
    }

    #[macro_export]
    macro_rules! test_main {
        ($($test_case:path),*$(,)?) => {
            fn main() {
                $crate::_internal::block_on(
                    $crate::_internal::run_tests(&[$(&$test_case),*])
                );
            }
        };
    }
}

/// Generate a test case.
pub use rye_macros::test_case;
