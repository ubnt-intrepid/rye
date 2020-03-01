use crate::reporter::TestCaseReporter;
use crate::test::TestResult;
use crate::{
    cli::{
        args::Args,
        exit_status::ExitStatus,
        report::{Outcome, OutcomeKind, Printer, Report},
    },
    executor::TestExecutor,
    test::{Registration, Registry, RegistryError, Test},
};
use futures::channel::oneshot;
use futures::stream::StreamExt as _;
use maybe_unwind::Unwind;
use std::collections::HashMap;
use std::error;
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
                let (tx, rx) = oneshot::channel();
                let reporter = SessionTestCaseReporter::new(tx);
                test.execute(&mut *executor, reporter);
                async {
                    let outcome = match rx.await {
                        Ok(Ok(())) => Outcome::passed(),
                        Ok(Err(msg)) => Outcome::failed().error_message(format!("{:?}", msg)),
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

#[derive(Debug)]
struct SessionTestCaseReporter {
    tx: Option<oneshot::Sender<Result<(), ErrorMessage>>>,
    failures: HashMap<String, String>,
}

impl SessionTestCaseReporter {
    fn new(tx: futures::channel::oneshot::Sender<Result<(), ErrorMessage>>) -> Self {
        Self {
            tx: Some(tx),
            failures: HashMap::new(),
        }
    }

    fn make_outcome(&mut self) -> Result<(), ErrorMessage> {
        if self.failures.is_empty() {
            Ok(())
        } else {
            Err(ErrorMessage(format!("{:?}", self).into()))
        }
    }
}

impl TestCaseReporter for SessionTestCaseReporter {
    fn test_case_starting(&mut self) {}

    fn test_case_ended(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(self.make_outcome());
        }
    }

    fn section_starting(&mut self, _: Option<&str>) {}

    fn section_ended(&mut self, name: Option<&str>, result: &dyn TestResult) {
        if !result.is_success() {
            let name = name.unwrap_or("__root__");
            self.failures.insert(
                name.into(),
                result
                    .error_message()
                    .map_or("<unknown>".into(), |msg| format!("{:?}", msg)),
            );
        }
    }

    fn section_terminated(&mut self, name: Option<&str>, unwind: &Unwind) {
        let name = name.unwrap_or("__root__");
        self.failures.insert(name.into(), unwind.to_string());
    }
}

struct ErrorMessage(Box<dyn error::Error + Send + Sync>);

impl fmt::Debug for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            fmt::Debug::fmt(&*self.0, f)
        } else {
            fmt::Display::fmt(&*self.0, f)
        }
    }
}
