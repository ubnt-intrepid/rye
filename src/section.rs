use phf::Set;

type SectionId = u64;

#[derive(Debug)]
pub struct Section {
    id: Option<SectionId>,
    #[allow(dead_code)]
    name: &'static str,
    is_leaf: bool,
    ancestors: Set<SectionId>,
}

impl Section {
    pub(crate) const ROOT: Self = Self {
        id: None,
        name: "root",
        is_leaf: true,
        ancestors: phf::phf_set!(),
    };

    #[doc(hidden)] // private API.
    pub const fn new(
        id: SectionId,
        name: &'static str,
        is_leaf: bool,
        ancestors: Set<SectionId>,
    ) -> Self {
        Self {
            id: Some(id),
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
        self.id
            .map_or(false, |t| t == id || self.ancestors.contains(&id))
    }
}
