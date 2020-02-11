use std::collections::HashSet;

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
