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
    unique_test_names: HashSet<String>,
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
            unique_test_names: HashSet::new(),
            _marker: PhantomData,
        })
    }

    pub(crate) fn register(
        &mut self,
        registration: &dyn Registration,
    ) -> Result<(), RegistryError> {
        registration.register(&mut MainRegistry { session: self })
    }

    pub(crate) fn sort_tests_by_names(&mut self) {
        self.pending_tests
            .sort_by(|t1, t2| t1.name().cmp(t2.name()));
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

struct MainRegistry<'a, 'sess> {
    session: &'a mut Session<'sess>,
}

impl Registry for MainRegistry<'_, '_> {
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
