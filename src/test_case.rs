use std::collections::{HashMap, HashSet};

/// Description about a test case.
#[derive(Debug)]
pub struct TestDesc {
    pub name: &'static str,
    pub module_path: &'static str,
    pub ignored: bool,
    pub sections: HashMap<SectionId, Section>,
    pub leaf_sections: &'static [SectionId],
}

pub(crate) type SectionId = u64;

#[derive(Debug)]
pub struct Section {
    #[allow(dead_code)]
    pub(crate) name: &'static str,
    pub(crate) ancestors: HashSet<SectionId>,
}

impl Section {
    #[doc(hidden)] // private API.
    pub const fn new(name: &'static str, ancestors: HashSet<SectionId>) -> Self {
        Self { name, ancestors }
    }
}
