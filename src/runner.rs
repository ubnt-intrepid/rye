use super::{
    cli::{Args, ExitStatus},
    executor::TestExecutor,
    registry::{register_all, Registration},
    report::{OutcomeKind, Printer, Report},
};
use futures::stream::StreamExt as _;
use std::{io::Write, sync::Once};

#[inline]
pub async fn run_tests<E: ?Sized>(tests: &[&dyn Registration], executor: &mut E) -> ExitStatus
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

    if args.list_tests() {
        let _ = printer.print_list(pending_tests.iter().map(|test| &test.desc));
        return ExitStatus::OK;
    }

    let _ = writeln!(printer.term(), "running {} tests", pending_tests.len());
    let name_length = pending_tests
        .iter()
        .map(|test| test.desc.test_name().len())
        .max()
        .unwrap_or(0);

    let completed_tests = {
        let completed_tests = futures::lock::Mutex::new(vec![]);
        futures::stream::iter(pending_tests)
            .for_each_concurrent(None, |test| {
                let handle = crate::executor::start_test(&test, &mut *executor);
                async {
                    let outcome = handle.await;
                    let _ = printer.print_result(&test.desc, name_length, &outcome);
                    completed_tests.lock().await.push((test, outcome));
                }
            })
            .await;
        completed_tests.into_inner()
    };

    let mut passed = vec![];
    let mut failed = vec![];
    for (test, outcome) in completed_tests {
        match outcome.kind() {
            OutcomeKind::Passed => passed.push(test.desc),
            OutcomeKind::Failed => failed.push((test.desc, outcome.err_msg())),
        }
    }

    let report = Report {
        passed,
        failed,
        filtered_out: filtered_out_tests
            .into_iter()
            .map(|test| test.desc)
            .collect(),
    };
    let _ = printer.print_report(&report);

    report.status()
}
