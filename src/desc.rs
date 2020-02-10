use crate::{context::TestContext, section::Section};
use futures::future::Future;

/// Metadata for executing a test case.
#[derive(Debug)]
pub struct TestDesc {
    pub name: &'static str,
    pub module_path: &'static str,
    pub sections: &'static [Section],
}

impl TestDesc {
    fn running_sections(&self) -> impl Iterator<Item = &Section> + '_ {
        self.sections.iter().filter(|section| section.is_leaf())
    }

    #[inline]
    pub fn run<F>(&self, f: F)
    where
        F: Fn(),
    {
        for section in self.running_sections() {
            TestContext::new(section).scope(&f);
        }
    }

    #[inline]
    pub async fn run_async<F, Fut>(&self, f: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = ()>,
    {
        for section in self.running_sections() {
            TestContext::new(section).scope_async(f()).await;
        }
    }
}
