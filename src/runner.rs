use super::{
    cli::{Args, ExitStatus},
    executor::TestExecutor,
    registry::{register_all, Registry, RegistryError},
    report::Report,
    report::{OutcomeKind, Printer},
};
use futures::stream::StreamExt as _;
use std::{io::Write, sync::Once};

#[inline]
pub async fn run_tests<E: ?Sized>(
    tests: &[&dyn Fn(&mut Registry<'_>) -> Result<(), RegistryError>],
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

    let (pending_tests, filtered_out_tests) = match register_all(tests, &args) {
        Ok(tests) => tests,
        Err(_) => return ExitStatus::FAILED,
    };

    let printer = Printer::new(&args);

    if args.list {
        printer.print_list(pending_tests.iter().map(|test| &test.desc));
        return ExitStatus::OK;
    }

    let _ = writeln!(printer.term(), "running {} tests", pending_tests.len());
    let name_length = pending_tests
        .iter()
        .map(|test| test.desc.name.len())
        .max()
        .unwrap_or(0);

    let completed_tests = {
        let completed_tests = futures::lock::Mutex::new(vec![]);
        futures::stream::iter(pending_tests)
            .for_each_concurrent(None, |test| {
                let handle = if args.run_ignored || !test.desc.ignored {
                    Some(crate::executor::start_test(&test, &mut *executor))
                } else {
                    None
                };
                async {
                    let outcome = match handle {
                        Some(handle) => Some(handle.await),
                        None => None,
                    };
                    printer.print_result(&test.desc, name_length, outcome.as_ref());
                    completed_tests.lock().await.push((test, outcome));
                }
            })
            .await;
        completed_tests.into_inner()
    };

    let mut passed = vec![];
    let mut failed = vec![];
    let mut measured = vec![];
    let mut ignored = vec![];
    for (test, outcome) in completed_tests {
        match outcome {
            Some(ref outcome) => match outcome.kind() {
                OutcomeKind::Passed => passed.push(test.desc),
                OutcomeKind::Failed => failed.push((test.desc, outcome.err_msg())),
                OutcomeKind::Measured { average, variance } => {
                    measured.push((test.desc, (*average, *variance)))
                }
            },
            None => ignored.push(test.desc),
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
    let _ = report.print(&printer);

    report.status()
}
