use crate::{context::TestContext, section::Section};

/// Metadata for executing a test case.
#[derive(Debug)]
pub struct TestCase {
    pub sections: &'static [Section],
}

impl TestCase {
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

    #[cfg(feature = "futures")]
    #[inline]
    pub async fn run_async<F, Fut>(&self, f: F)
    where
        F: Fn() -> Fut,
        Fut: futures_core::Future<Output = ()>,
    {
        for section in self.running_sections() {
            TestContext::new(section).scope_async(f()).await;
        }
    }
}
