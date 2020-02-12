use crate::{
    context::TestContext,
    section::{Section, SectionId},
};
use futures::future::Future;
use std::collections::HashMap;

/// Metadata for executing a test case.
#[derive(Debug)]
pub struct TestDesc {
    pub name: &'static str,
    pub module_path: &'static str,
    pub ignored: bool,
    pub sections: HashMap<SectionId, Section>,
    pub leaf_sections: &'static [SectionId],
}

impl TestDesc {
    #[inline]
    pub fn run<F>(&self, f: F)
    where
        F: Fn(),
    {
        if self.leaf_sections.is_empty() {
            TestContext::new(self, None).scope(&f);
        } else {
            for &section in self.leaf_sections {
                TestContext::new(self, Some(section)).scope(&f);
            }
        }
    }

    #[inline]
    pub async fn run_async<F, Fut>(&self, f: F)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = ()>,
    {
        if self.leaf_sections.is_empty() {
            TestContext::new(self, None).scope_async(f()).await;
        } else {
            for &section in self.leaf_sections {
                TestContext::new(self, Some(section)).scope_async(f()).await;
            }
        }
    }
}
