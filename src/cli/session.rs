use crate::{
    cli::{args::Args, exit_status::ExitStatus},
    executor::TestExecutor,
    reporter::Reporter,
    test::{Registration, Registry, RegistryError, Test},
};
use std::{
    collections::HashSet,
    io::{self, Write as _},
};

pub struct Session<'sess> {
    args: &'sess Args,
    pending_tests: Vec<Test>,
    filtered_out_tests: Vec<Test>,
    unique_test_names: HashSet<String>,
}

impl<'sess> Session<'sess> {
    #[inline]
    pub fn new(args: &'sess Args) -> Self {
        Self {
            args,
            pending_tests: vec![],
            filtered_out_tests: vec![],
            unique_test_names: HashSet::new(),
        }
    }

    fn print_list(&self) -> io::Result<()> {
        let term = io::stdout();
        let mut term = term.lock();
        let mut num_tests = 0;

        for test in &self.pending_tests {
            num_tests += 1;
            writeln!(term, "{}: test", test.desc().name())?;
        }

        fn plural_suffix(n: usize) -> &'static str {
            match n {
                1 => "",
                _ => "s",
            }
        }

        if num_tests != 0 {
            writeln!(term)?;
        }
        writeln!(term, "{} test{}", num_tests, plural_suffix(num_tests),)?;

        term.flush()?;
        Ok(())
    }

    #[inline]
    pub fn run<E: ?Sized, R: ?Sized>(
        &mut self,
        tests: &[&dyn Registration],
        executor: &mut E,
        reporter: &mut R,
    ) -> ExitStatus
    where
        E: TestExecutor,
        R: Reporter + Send + Clone + 'static,
    {
        for &test in tests {
            let res = test.register(&mut MainRegistry { session: self });
            if let Err(err) = res {
                eprintln!("registry error: {}", err);
                return ExitStatus::FAILED;
            }
        }

        // sort test cases by name.
        self.pending_tests
            .sort_by(|t1, t2| t1.desc().name().cmp(t2.desc().name()));

        if self.args.list_tests {
            let _ = self.print_list();
            return ExitStatus::OK;
        }

        reporter.test_run_starting(&self.pending_tests);

        for test in self.pending_tests.drain(..) {
            let reporter = reporter.clone();
            test.execute(&mut *executor, reporter);
        }

        let mut summary = executor.run();
        summary.filtered_out = self
            .filtered_out_tests
            .iter()
            .map(|test| test.desc())
            .collect();

        reporter.test_run_ended(&summary);

        if summary.is_success() {
            ExitStatus::OK
        } else {
            ExitStatus::FAILED
        }
    }
}

struct MainRegistry<'a, 'sess> {
    session: &'a mut Session<'sess>,
}

impl Registry for MainRegistry<'_, '_> {
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        let session = &mut *self.session;

        if !session.unique_test_names.insert(test.desc().name().into()) {
            return Err(RegistryError::new(format!(
                "the test name '{}' is conflicted",
                test.desc().name()
            )));
        }

        if session.args.is_filtered_out(test.desc().name()) {
            session.filtered_out_tests.push(test);
        } else {
            session.pending_tests.push(test);
        }

        Ok(())
    }
}
