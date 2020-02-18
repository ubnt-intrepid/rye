use crate::{cli::Args, test::Test};
use std::collections::HashSet;

pub struct RegistryError(());

pub struct Registry<'a> {
    args: &'a Args,
    inner: &'a mut RegistryInner,
}

#[derive(Default)]
struct RegistryInner {
    pending_tests: Vec<Test>,
    filtered_out_tests: Vec<Test>,
    unique_test_names: HashSet<String>,
}

impl Registry<'_> {
    #[doc(hidden)] // private API
    pub fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        if !self
            .inner
            .unique_test_names
            .insert(test.desc.name.to_string())
        {
            eprintln!("the test name is conflicted: {}", test.desc.name);
            return Err(RegistryError(()));
        }

        if self.args.is_match(test.desc.name) {
            self.inner.pending_tests.push(test);
        } else {
            self.inner.filtered_out_tests.push(test);
        }

        Ok(())
    }
}

pub(crate) fn register_all(
    tests: &[&dyn Fn(&mut Registry<'_>) -> Result<(), RegistryError>],
    args: &Args,
) -> Result<(Vec<Test>, Vec<Test>), RegistryError> {
    let mut inner = RegistryInner::default();
    for test in tests {
        test(&mut Registry {
            args,
            inner: &mut inner,
        })?;
    }
    Ok((inner.pending_tests, inner.filtered_out_tests))
}
