use crate::section::{Section, Sections};

#[derive(Debug)]
#[must_use]
pub struct TestCase {
    sections: Sections,
}

impl Default for TestCase {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TestCase {
    /// Create a test case.
    pub fn new() -> Self {
        Self {
            sections: Sections::new(),
        }
    }

    pub fn completed(&self) -> bool {
        self.sections.completed()
    }

    pub fn root_section(&self) -> Section {
        self.sections.root()
    }
}
