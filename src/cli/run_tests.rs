use crate::{
    cli::{exit_status::ExitStatus, session::Session},
    test::Registration,
};
use std::io::Write as _;

#[inline]
pub fn run_tests<F>(tests: &[&dyn Registration], f: F)
where
    F: FnOnce(&mut Session),
{
    run_tests_inner(tests, f).exit();
}

fn run_tests_inner<F>(tests: &[&dyn Registration], f: F) -> ExitStatus
where
    F: FnOnce(&mut Session),
{
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
