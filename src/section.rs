use phf::Set;

pub(crate) type SectionId = u64;

#[derive(Debug)]
pub struct Section {
    id: SectionId,
    #[allow(dead_code)]
    name: &'static str,
    is_leaf: bool,
    ancestors: Set<SectionId>,
}

impl Section {
    #[doc(hidden)] // private API.
    pub const fn new(
        id: SectionId,
        name: &'static str,
        is_leaf: bool,
        ancestors: Set<SectionId>,
    ) -> Self {
        Self {
            id,
            name,
            is_leaf,
            ancestors,
        }
    }

    #[doc(hidden)] // private API.
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    #[inline]
    pub fn is_target(&self, id: SectionId) -> bool {
        self.id == id || self.ancestors.contains(&id)
    }
}
