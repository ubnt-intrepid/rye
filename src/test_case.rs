use crate::section::{SectionData, SectionId, SectionState};
use std::{cell::Cell, collections::hash_map::HashMap, ptr::NonNull};

thread_local! {
    static TEST_CASE: Cell<Option<NonNull<TestCase>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<TestCase>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        TEST_CASE.with(|tls| tls.set(self.0.take()));
    }
}

#[derive(Debug)]
pub(crate) struct TestCase {
    pub(crate) sections: HashMap<SectionId, SectionData>,
}

#[allow(clippy::new_without_default)]
impl TestCase {
    /// Create a test case.
    pub(crate) fn new(name: &'static str) -> Self {
        let mut sections = HashMap::new();
        sections.insert(
            SectionId::root(),
            SectionData {
                name,
                state: SectionState::Found,
                children: vec![],
            },
        );
        Self { sections }
    }

    pub(crate) fn completed(&self) -> bool {
        let root = &self.sections[&SectionId::root()];
        root.state == SectionState::Completed
    }

    pub(crate) fn scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let prev = TEST_CASE.with(|tls| tls.replace(Some(NonNull::from(self))));
        let _reset = SetOnDrop(prev);
        f()
    }

    pub(crate) fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut TestCase) -> R,
    {
        let test_case_ptr = TEST_CASE.with(|tls| tls.take());
        let _reset = SetOnDrop(test_case_ptr);
        let mut test_case_ptr = test_case_ptr.expect("test case is not set on the current thread");
        unsafe { f(test_case_ptr.as_mut()) }
    }
}
