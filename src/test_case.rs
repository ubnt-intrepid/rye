use crate::section::{Section, SectionData, SectionId, SectionState};
use std::{cell::RefCell, collections::hash_map::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub(crate) struct TestCase {
    pub(crate) sections: Rc<RefCell<HashMap<&'static SectionId, SectionData>>>,
}

#[allow(clippy::new_without_default)]
impl TestCase {
    /// Create a test case.
    pub(crate) fn new() -> Self {
        let mut sections = HashMap::new();
        sections.insert(
            &SectionId::Root,
            SectionData {
                state: SectionState::Found,
                children: vec![],
            },
        );
        Self {
            sections: Rc::new(RefCell::new(sections)),
        }
    }

    pub(crate) fn completed(&self) -> bool {
        let sections = self.sections.borrow();
        let root = &sections[&SectionId::Root];
        root.state == SectionState::Completed
    }

    pub(crate) fn root_section(&self) -> Section {
        Section {
            test_case: self.clone(),
            id: &SectionId::Root,
            encounted: false,
        }
    }
}
