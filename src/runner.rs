use crate::{
    cli::ExitStatus, executor::DefaultTestExecutor, registry::Registration, session::Session,
};
use futures::executor::block_on;
use std::{io::Write, sync::Once};

pub fn default_runner(tests: &[&dyn Registration]) {
    run_tests(tests, |session| {
        let mut executor = DefaultTestExecutor::new().unwrap();
        block_on(session.run_tests_concurrent(&mut executor));
    })
    .exit();
}

fn run_tests<F>(tests: &[&dyn Registration], f: F) -> ExitStatus
where
    F: FnOnce(&mut Session),
{
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        maybe_unwind::set_hook();
    });

    let mut session = match Session::from_env() {
        Ok(sess) => sess,
        Err(st) => return st,
    };

    if let Err(st) = session.register(tests) {
        return st;
    };

    if session.args.list_tests() {
        let _ = session
            .printer
            .print_list(session.pending_tests.iter().map(|test| &test.desc));
        return ExitStatus::OK;
    }

    let _ = writeln!(
        session.printer.term(),
        "running {} tests",
        session.pending_tests.len()
    );
    f(&mut session);

    let report = session.make_report();
    let _ = session.printer.print_report(&report);
    report.status()
}
