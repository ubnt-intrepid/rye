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
use std::{collections::HashSet, fmt, marker::PhantomData};

pub struct Session<'a> {
    pub(crate) args: Args,
    pub(crate) printer: Printer,
    pub(crate) pending_tests: Vec<Test>,
    pub(crate) filtered_out_tests: Vec<Test>,
    pub(crate) completed_tests: Vec<(Test, Outcome)>,
    #[allow(clippy::type_complexity)]
    _marker: PhantomData<(fn(&'a ()) -> &'a (), std::rc::Rc<std::cell::Cell<()>>)>,
}

impl Session<'_> {
    pub(crate) fn from_env() -> Result<Self, ExitStatus> {
        let args = Args::from_env()?;
        let printer = Printer::new(&args);
        Ok(Self {
            args,
            printer,
            pending_tests: vec![],
            filtered_out_tests: vec![],
            completed_tests: vec![],
            _marker: PhantomData,
        })
    }

    pub(crate) fn register(&mut self, tests: &[&dyn Registration]) -> Result<(), ExitStatus> {
        let (pending_tests, filtered_out_tests) = match register_all(tests, &self.args) {
            Ok(tests) => tests,
            Err(err) => {
                eprintln!("registry error: {}", err);
                return Err(ExitStatus::FAILED);
            }
        };
        self.pending_tests = pending_tests;
        self.filtered_out_tests = filtered_out_tests;
        Ok(())
    }

    /// Execute test case onto the specified executor.
    pub async fn execute_tests<'a, E: ?Sized>(&'a mut self, executor: &'a mut E)
    where
        E: TestExecutor,
        E::Handle: TryFuture<Ok = ()>,
        <E::Handle as TryFuture>::Error: fmt::Debug,
    {
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
    }

    pub(crate) fn make_report(&mut self) -> Report {
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
    args: &'a Args,
    inner: &'a mut MainRegistryInner,
}

#[derive(Default)]
struct MainRegistryInner {
    pending_tests: Vec<Test>,
    filtered_out_tests: Vec<Test>,
    unique_test_names: HashSet<String>,
}

impl Registry for MainRegistry<'_> {
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        if !self.inner.unique_test_names.insert(test.name().into()) {
            return Err(RegistryError::new(format!(
                "the test name '{}' is conflicted",
                test.name()
            )));
        }

        if self.args.is_match(test.name()) {
            self.inner.pending_tests.push(test);
        } else {
            self.inner.filtered_out_tests.push(test);
        }

        Ok(())
    }
}

fn register_all(
    registrations: &[&dyn Registration],
    args: &Args,
) -> Result<(Vec<Test>, Vec<Test>), RegistryError> {
    let mut inner = MainRegistryInner::default();
    for registration in registrations {
        registration.register(&mut MainRegistry {
            args,
            inner: &mut inner,
        })?;
    }
    Ok((inner.pending_tests, inner.filtered_out_tests))
}
