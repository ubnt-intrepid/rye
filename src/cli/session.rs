use crate::{
    cli::{
        args::Args,
        exit_status::ExitStatus,
        report::{Outcome, OutcomeKind, Printer, Report},
    },
    executor::TestExecutor,
    test::{Registration, Registry, RegistryError, Test},
};
use futures::{
    future::{TryFuture, TryFutureExt as _},
    stream::StreamExt as _,
};
use std::{collections::HashSet, fmt, io::Write as _};

pub struct Session {
    args: Args,
    printer: Printer,
    pending_tests: Vec<Test>,
    filtered_out_tests: Vec<Test>,
    completed_tests: Vec<(Test, Outcome)>,
    unique_test_names: HashSet<String>,
}

impl Session {
    #[inline]
    pub fn from_env(tests: &[&dyn Registration]) -> Self {
        let args = Args::from_env().unwrap_or_else(|st| st.exit());
        let printer = Printer::new(&args);
        let mut sess = Self {
            args,
            printer,
            pending_tests: vec![],
            filtered_out_tests: vec![],
            completed_tests: vec![],
            unique_test_names: HashSet::new(),
        };

        for &test in tests {
            let res = test.register(&mut MainRegistry { session: &mut sess });
            if let Err(err) = res {
                eprintln!("registry error: {}", err);
                ExitStatus::FAILED.exit();
            }
        }

        // sort test cases by name.
        sess.pending_tests
            .sort_by(|t1, t2| t1.name().cmp(t2.name()));

        sess
    }

    #[inline]
    pub async fn run<'a, E: ?Sized>(&'a mut self, executor: &'a mut E) -> ExitStatus
    where
        E: TestExecutor,
        E::Handle: TryFuture<Ok = ()>,
        <E::Handle as TryFuture>::Error: fmt::Debug,
    {
        if self.args.list_tests {
            let _ = self.printer.print_list(self.pending_tests.iter());
            return ExitStatus::OK;
        }

        let _ = writeln!(
            self.printer.term(),
            "running {} tests",
            self.pending_tests.len()
        );

        let name_length = self
            .pending_tests
            .iter()
            .map(|test| test.name().len())
            .max()
            .unwrap_or(0);

        let completed_tests = futures::lock::Mutex::new(vec![]);
        let printer = &self.printer;
        futures::stream::iter(self.pending_tests.drain(..))
            .for_each_concurrent(None, |test| {
                let handle = test.execute(&mut *executor);
                async {
                    let outcome = match handle.into_future().await {
                        Ok(()) => Outcome::passed(),
                        Err(err) => Outcome::failed().error_message(format!("{:?}", err)),
                    };
                    let _ = printer.print_result(&test, name_length, &outcome);
                    completed_tests.lock().await.push((test, outcome));
                }
            })
            .await;

        self.completed_tests = completed_tests.into_inner();

        let report = self.make_report();
        let _ = self.printer.print_report(&report);

        report.status()
    }

    fn make_report(&mut self) -> Report {
        let mut passed = vec![];
        let mut failed = vec![];
        for (test, outcome) in self.completed_tests.drain(..) {
            match outcome.kind() {
                OutcomeKind::Passed => passed.push(test),
                OutcomeKind::Failed => failed.push((test, outcome.err_msg())),
            }
        }
        Report {
            passed,
            failed,
            filtered_out: self.filtered_out_tests.drain(..).collect(),
        }
    }
}

struct MainRegistry<'a> {
    session: &'a mut Session,
}

impl Registry for MainRegistry<'_> {
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        let session = &mut *self.session;

        if !session.unique_test_names.insert(test.name().into()) {
            return Err(RegistryError::new(format!(
                "the test name '{}' is conflicted",
                test.name()
            )));
        }

        if session.args.is_filtered_out(test.name()) {
            session.filtered_out_tests.push(test);
        } else {
            session.pending_tests.push(test);
        }

        Ok(())
    }
}
