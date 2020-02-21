use crate::{cli::ExitStatus, executor::DefaultTestExecutor, session::Session};
use futures::executor::LocalPool;
use rye::_internal::Registration;
use std::{io::Write, sync::Once};

pub fn runner(tests: &[&dyn Registration]) {
    run_tests(tests, |session| {
        let mut local_pool = LocalPool::new();
        let mut executor = DefaultTestExecutor::new(local_pool.spawner()).unwrap();
        local_pool.run_until(session.run_tests_concurrent(&mut executor));
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
        let _ = session.printer.print_list(session.pending_tests.iter());
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
