use crate::{
    cli::{args::Args, exit_status::ExitStatus},
    executor::TestExecutor,
    reporter::console::{ConsoleReporter, ConsoleTestCaseReporter, Outcome, OutcomeKind, Report},
    test::{Registration, Registry, RegistryError, Test},
};
use futures::channel::oneshot;
use futures::stream::StreamExt as _;
use std::error;
use std::{collections::HashSet, io::Write as _};

pub struct Session<'sess> {
    args: &'sess Args,
    pending_tests: Vec<Test>,
    filtered_out_tests: Vec<Test>,
    completed_tests: Vec<(Test, Outcome)>,
    unique_test_names: HashSet<String>,
}

impl<'sess> Session<'sess> {
    #[inline]
    pub fn new(args: &'sess Args) -> Self {
        Self {
            args,
            pending_tests: vec![],
            filtered_out_tests: vec![],
            completed_tests: vec![],
            unique_test_names: HashSet::new(),
        }
    }

    pub fn register_tests(
        &mut self,
        tests: &[&dyn Registration],
    ) -> Result<(), impl error::Error + Send + Sync + 'static> {
        for &test in tests {
            let res = test.register(&mut MainRegistry { session: self });
            if let Err(err) = res {
                return Err(err);
            }
        }

        // sort test cases by name.
        self.pending_tests
            .sort_by(|t1, t2| t1.desc().name().cmp(t2.desc().name()));

        Ok(())
    }

    #[inline]
    pub async fn run<'a, E: ?Sized>(
        &'a mut self,
        executor: &'a mut E,
        printer: &mut ConsoleReporter,
    ) -> ExitStatus
    where
        E: TestExecutor,
    {
        if self.args.list_tests {
            let _ = printer.print_list(self.pending_tests.iter().map(|test| test.desc()));
            return ExitStatus::OK;
        }

        let _ = writeln!(printer.term(), "running {} tests", self.pending_tests.len());

        let name_length = self
            .pending_tests
            .iter()
            .map(|test| test.desc().name().len())
            .max()
            .unwrap_or(0);

        let completed_tests = futures::lock::Mutex::new(vec![]);
        let printer = &*printer;
        futures::stream::iter(self.pending_tests.drain(..))
            .for_each_concurrent(None, |test| {
                let (tx, rx) = oneshot::channel();
                let reporter = ConsoleTestCaseReporter::new(tx);
                test.execute(&mut *executor, reporter);
                async {
                    let outcome = match rx.await {
                        Ok(Ok(())) => Outcome::passed(),
                        Ok(Err(msg)) => Outcome::failed().error_message(format!("{:?}", msg)),
                        Err(err) => Outcome::failed().error_message(format!("{:?}", err)),
                    };
                    let _ = printer.print_result(test.desc(), name_length, &outcome);
                    completed_tests.lock().await.push((test, outcome));
                }
            })
            .await;

        self.completed_tests = completed_tests.into_inner();

        let report = self.make_report();
        let _ = printer.print_report(&report);

        report.status()
    }

    fn make_report(&mut self) -> Report {
        let mut passed = vec![];
        let mut failed = vec![];
        for (test, outcome) in self.completed_tests.drain(..) {
            match outcome.kind() {
                OutcomeKind::Passed => passed.push(test.desc()),
                OutcomeKind::Failed => failed.push((test.desc(), outcome.err_msg())),
            }
        }
        Report {
            passed,
            failed,
            filtered_out: self
                .filtered_out_tests
                .drain(..)
                .map(|test| test.desc())
                .collect(),
        }
    }
}

struct MainRegistry<'a, 'sess> {
    session: &'a mut Session<'sess>,
}

impl Registry for MainRegistry<'_, '_> {
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        let session = &mut *self.session;

        if !session.unique_test_names.insert(test.desc().name().into()) {
            return Err(RegistryError::new(format!(
                "the test name '{}' is conflicted",
                test.desc().name()
            )));
        }

        if session.args.is_filtered_out(test.desc().name()) {
            session.filtered_out_tests.push(test);
        } else {
            session.pending_tests.push(test);
        }

        Ok(())
    }
}
