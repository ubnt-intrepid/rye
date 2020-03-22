#![allow(missing_docs)]

use crate::{
    args::Args,
    executor::{TestExecutor, TestExecutorExt as _},
    exit_status::ExitStatus,
    reporter::{Reporter, Summary},
    test::TestCase,
};
use hashbrown::HashSet;

pub struct Session<'sess> {
    args: &'sess Args,
}

impl<'sess> Session<'sess> {
    #[inline]
    pub fn new(args: &'sess Args) -> Self {
        Self { args }
    }

    #[inline]
    pub async fn run<T: ?Sized, R: ?Sized>(
        &mut self,
        tests: &[&dyn TestCase],
        runner: &mut T,
        reporter: &R,
    ) -> ExitStatus
    where
        T: TestExecutor,
        R: Reporter + Send + Clone + 'static,
    {
        let mut registered_tests = vec![];
        let mut filtered_out_tests = vec![];
        let mut unique_test_names = HashSet::new();
        for test in tests {
            let desc = test.desc();
            let filtered_out = self.args.is_filtered_out(desc.name());

            if !unique_test_names.insert(desc.name().to_owned()) {
                eprintln!("the test name '{}' is conflicted", desc.name());
                return ExitStatus::FAILED;
            }

            if filtered_out {
                filtered_out_tests.push(desc);
            } else {
                registered_tests.push(*test);
            }
        }

        // sort test cases by name.
        registered_tests.sort_by(|t1, t2| t1.desc().name().cmp(t2.desc().name()));

        if self.args.list_tests {
            let mut num_tests = 0;
            for test in &registered_tests {
                num_tests += 1;
                println!("{}: test", test.desc().name());
            }

            fn plural_suffix(n: usize) -> &'static str {
                match n {
                    1 => "",
                    _ => "s",
                }
            }

            if num_tests != 0 {
                println!();
            }
            println!("{} test{}", num_tests, plural_suffix(num_tests));

            return ExitStatus::OK;
        }

        reporter.test_run_starting(&registered_tests[..]);

        let mut summary = Summary::empty();
        summary.filtered_out.extend(filtered_out_tests);
        let mut handles = vec![];
        for test in registered_tests.drain(..) {
            let reporter = reporter.clone();
            handles.push(runner.spawn_test(test, reporter));
        }
        let results = futures::future::join_all(handles).await;
        for result in results {
            summary.append(result);
        }

        reporter.test_run_ended(&summary);

        if summary.is_passed() {
            ExitStatus::OK
        } else {
            ExitStatus::FAILED
        }
    }
}
