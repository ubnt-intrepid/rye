#![allow(missing_docs)]

use crate::{
    args::Args,
    executor::{TestExecutor, TestExecutorExt as _},
    exit_status::ExitStatus,
    reporter::{Reporter, Summary},
    test::{imp::TestFn, Registry, RegistryError, Test, TestDesc, TestSet},
};
use std::{
    collections::HashSet,
    io::{self, Write as _},
    sync::Arc,
};

pub struct Session<'sess> {
    args: &'sess Args,
    registered_tests: Vec<Test>,
    unique_test_names: HashSet<String>,
}

impl<'sess> Session<'sess> {
    #[inline]
    pub fn new(args: &'sess Args) -> Self {
        Self {
            args,
            registered_tests: vec![],
            unique_test_names: HashSet::new(),
        }
    }

    fn print_list(&self) -> io::Result<()> {
        let term = io::stdout();
        let mut term = term.lock();
        let mut num_tests = 0;

        for test in self
            .registered_tests
            .iter()
            .filter(|test| !test.filtered_out)
        {
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
    pub fn run<T: ?Sized, R: ?Sized>(
        &mut self,
        tests: &[&dyn TestSet],
        runner: &mut T,
        reporter: &R,
    ) -> ExitStatus
    where
        T: TestExecutor,
        R: Reporter + Send + Clone + 'static,
    {
        let res = tests.register(&mut MainRegistry { session: self });
        if let Err(err) = res {
            eprintln!("registry error: {}", err);
            return ExitStatus::FAILED;
        }

        // sort test cases by name.
        self.registered_tests
            .sort_by(|t1, t2| t1.desc().name().cmp(t2.desc().name()));

        if self.args.list_tests {
            let _ = self.print_list();
            return ExitStatus::OK;
        }

        reporter.test_run_starting(&self.registered_tests);

        let mut summary = Summary::empty();
        let mut handles = vec![];
        for test in self.registered_tests.drain(..) {
            if test.filtered_out {
                summary.filtered_out.push(test.desc);
            } else {
                let reporter = reporter.clone();
                handles.push(runner.spawn_test(&test, reporter));
            }
        }
        runner.run(async {
            let results = futures::future::join_all(handles).await;
            for result in results {
                summary.append(result);
            }
        });

        reporter.test_run_ended(&summary);

        if summary.is_passed() {
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
    fn add_test(&mut self, desc: TestDesc, test_fn: TestFn) -> Result<(), RegistryError> {
        let session = &mut *self.session;
        let filtered_out = session.args.is_filtered_out(desc.name());

        if !session.unique_test_names.insert(desc.name().into()) {
            return Err(RegistryError::new(format!(
                "the test name '{}' is conflicted",
                desc.name()
            )));
        }

        session.registered_tests.push(Test {
            desc: Arc::new(desc),
            test_fn,
            filtered_out,
        });

        Ok(())
    }
}
