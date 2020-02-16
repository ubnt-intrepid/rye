use super::{
    args::Args,
    context::TestContext,
    executor::TestExecutor,
    exit_status::ExitStatus,
    outcome::{Outcome, OutcomeKind},
    printer::Printer,
    report::Report,
};
use crate::test_case::{TestCase, TestFn};
use expected::{expected, Disappoints, FutureExpectedExt as _};
use futures::{
    future::Future,
    ready,
    stream::StreamExt as _,
    task::{self, Poll},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use pin_project::pin_project;
use std::{collections::HashSet, io::Write, panic::AssertUnwindSafe, pin::Pin};

pub(crate) struct TestDriver<'a> {
    args: &'a Args,
    printer: Printer,
}

impl<'a> TestDriver<'a> {
    pub(crate) fn new(args: &'a Args) -> Self {
        let printer = Printer::new(&args);
        Self { args, printer }
    }

    pub(crate) async fn run_tests<E: ?Sized>(
        &mut self,
        test_cases: impl IntoIterator<Item = TestCase>,
        executor: &mut E,
    ) -> Result<Report, ExitStatus>
    where
        E: TestExecutor,
    {
        // First, convert each test case to PendingTest for tracking the running state.
        // Test cases that satisfy the skip condition are filtered out here.
        let mut pending_tests = vec![];
        let mut filtered_out_tests = vec![];
        let mut unique_test_names = HashSet::new();
        for test in test_cases {
            if !unique_test_names.insert(test.desc.name.to_string()) {
                let _ = writeln!(
                    self.printer.term(),
                    "the test name is conflicted: {}",
                    test.desc.name
                );
                return Err(ExitStatus::FAILED);
            }

            if self.args.is_filtered(test.desc.name) {
                filtered_out_tests.push(test);
                continue;
            }

            // Since PendingTest may contain the immovable state must be pinned
            // before starting any operations.
            // Here, each test case is allocated on the heap.
            pending_tests.push(Box::pin(PendingTest {
                test_case: test,
                handle: None,
                outcome: None,
                printer: &self.printer,
                name_length: 0,
            }));
        }

        if self.args.list {
            self.printer
                .print_list(pending_tests.iter().map(|test| &test.test_case.desc));
            return Err(ExitStatus::OK);
        }

        let _ = writeln!(self.printer.term(), "running {} tests", pending_tests.len());

        let max_name_length = pending_tests
            .iter()
            .map(|test| test.test_case.desc.name.len())
            .max()
            .unwrap_or(0);

        futures::stream::iter(pending_tests.iter_mut())
            .for_each_concurrent(None, |pending_test| {
                pending_test
                    .as_mut()
                    .start(&self.args, max_name_length, &mut *executor);
                pending_test
            })
            .await;

        let mut passed = vec![];
        let mut failed = vec![];
        let mut measured = vec![];
        let mut ignored = vec![];
        for test in &pending_tests {
            match test.outcome {
                Some(ref outcome) => match outcome.kind() {
                    OutcomeKind::Passed => passed.push(test.test_case.desc.clone()),
                    OutcomeKind::Failed => {
                        failed.push((test.test_case.desc.clone(), outcome.err_msg()))
                    }
                    OutcomeKind::Measured { average, variance } => {
                        measured.push((test.test_case.desc.clone(), (*average, *variance)))
                    }
                },
                None => ignored.push(test.test_case.desc.clone()),
            }
        }

        let report = Report {
            passed,
            failed,
            measured,
            ignored,
            filtered_out: filtered_out_tests
                .into_iter()
                .map(|test| test.desc)
                .collect(),
        };
        let _ = report.print(&self.printer);

        Ok(report)
    }
}

#[pin_project]
struct PendingTest<'a, T> {
    test_case: TestCase,
    #[pin]
    handle: Option<T>,
    outcome: Option<Outcome>,
    printer: &'a Printer,
    name_length: usize,
}

impl<T> PendingTest<'_, T>
where
    T: Future<Output = Outcome>,
{
    fn start<E: ?Sized>(self: Pin<&mut Self>, args: &Args, name_length: usize, executor: &mut E)
    where
        E: TestExecutor<Handle = T>,
    {
        let mut me = self.project();

        *me.name_length = name_length;

        let ignored = (me.test_case.desc.ignored && !args.run_ignored) || !args.run_tests;

        if !ignored {
            let handle = {
                let desc = me.test_case.desc.clone();
                match me.test_case.test_fn {
                    TestFn::SyncTest(f) => executor.execute_blocking(move || {
                        let res = expected(|| {
                            maybe_unwind(AssertUnwindSafe(|| {
                                if desc.leaf_sections.is_empty() {
                                    TestContext::new(&desc, None).scope(&f);
                                } else {
                                    for &section in desc.leaf_sections {
                                        TestContext::new(&desc, Some(section)).scope(&f);
                                    }
                                }
                            }))
                        });
                        make_outcome(res)
                    }),
                    TestFn::AsyncTest(f) => executor.execute(async move {
                        let res = AssertUnwindSafe(async move {
                            if desc.leaf_sections.is_empty() {
                                TestContext::new(&desc, None).scope_async(f()).await;
                            } else {
                                for &section in desc.leaf_sections {
                                    TestContext::new(&desc, Some(section))
                                        .scope_async(f())
                                        .await;
                                }
                            }
                        })
                        .maybe_unwind()
                        .expected()
                        .await;
                        make_outcome(res)
                    }),
                }
            };
            me.handle.set(Some(handle));
        }
    }
}

impl<T> Future for PendingTest<'_, T>
where
    T: Future<Output = Outcome>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();

        match me.handle.as_pin_mut() {
            Some(handle) => {
                let outcome = ready!(handle.poll(cx));
                me.printer
                    .print_result(&me.test_case.desc, *me.name_length, Some(&outcome));
                me.outcome.replace(outcome);
            }
            None => {
                me.printer
                    .print_result(&me.test_case.desc, *me.name_length, None);
            }
        }

        Poll::Ready(())
    }
}

fn make_outcome(res: (Result<(), Unwind>, Option<Disappoints>)) -> Outcome {
    match res {
        (Ok(()), None) => Outcome::passed(),
        (Ok(()), Some(disappoints)) => Outcome::failed().error_message(disappoints.to_string()),
        (Err(unwind), disappoints) => {
            use std::fmt::Write as _;
            let mut msg = String::new();
            let _ = writeln!(&mut msg, "{}", unwind);
            if let Some(disappoints) = disappoints {
                let _ = writeln!(&mut msg, "{}", disappoints);
            }
            Outcome::failed().error_message(msg)
        }
    }
}