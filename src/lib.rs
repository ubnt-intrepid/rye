/*!
A custom unit testing framework inspired by Catch2.
!*/

mod args;
mod context;
mod driver;
mod executor;
mod exit_status;
mod outcome;
mod printer;
mod report;
mod test_case;
mod test_suite;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        executor::{DefaultTestExecutor, TestExecutor},
        exit_status::ExitStatus,
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

    #[inline]
    pub async fn run_tests<E: ?Sized>(
        tests: &[&dyn Fn(&mut TestSuite<'_>)],
        executor: &mut E,
    ) -> ExitStatus
    where
        E: TestExecutor,
    {
        let args = match Args::from_env() {
            Ok(args) => args,
            Err(st) => return st,
        };

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
        match driver.run_tests(test_cases, &mut *executor).await {
            Ok(report) => report.status(),
            Err(status) => status,
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

/// Generate a test case.
pub use rye_macros::test_case;
