use pin_project::{pin_project, pinned_drop};
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    fmt,
    marker::PhantomPinned,
    pin::Pin,
    rc::Rc,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SectionId {
    Root,
    SubSection {
        name: &'static str,
        file: &'static str,
        line: u32,
        column: u32,
    },
}

impl fmt::Debug for SectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root => f.write_str("<root>"),
            Self::SubSection {
                name,
                file,
                line,
                column,
            } => write!(f, "[{}:{}:{}:{}]", name, file, line, column),
        }
    }
}

#[derive(Debug)]
struct SectionData {
    state: SectionState,
    children: Vec<&'static SectionId>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SectionState {
    Found,
    Completed,
}

#[pin_project(PinnedDrop)]
pub struct Section {
    sections: Sections,
    id: &'static SectionId,
    encounted: bool,
    _marker: PhantomPinned,
}

impl Section {
    pub(crate) fn new_section(self: Pin<&mut Self>, id: &'static SectionId) -> Option<Section> {
        let me = self.project();

        let mut sections = me.sections.inner.borrow_mut();
        let insert_child;
        let is_target;
        match sections.entry(id) {
            Entry::Occupied(entry) => {
                let data = entry.into_mut();
                match data.state {
                    SectionState::Found if !*me.encounted => {
                        *me.encounted = true;
                        insert_child = false;
                        is_target = true;
                    }
                    _ => {
                        insert_child = false;
                        is_target = false;
                    }
                }
            }
            Entry::Vacant(entry) => {
                if *me.encounted {
                    entry.insert(SectionData {
                        state: SectionState::Found,
                        children: vec![],
                    });
                    insert_child = true;
                    is_target = false;
                } else {
                    *me.encounted = true;
                    entry.insert(SectionData {
                        state: SectionState::Found,
                        children: vec![],
                    });

                    insert_child = true;
                    is_target = true;
                }
            }
        }
        if insert_child {
            sections.get_mut(&*me.id).unwrap().children.push(id);
        }

        if is_target {
            Some(Section {
                sections: me.sections.clone(),
                id,
                encounted: false,
                _marker: PhantomPinned,
            })
        } else {
            None
        }
    }

    fn check_completed(self: Pin<&mut Self>) -> Result<(), ()> {
        let mut sections = self.sections.inner.try_borrow_mut().map_err(drop)?;

        let mut completed = true;
        let data = sections.get(&self.id).ok_or(())?;
        for child in &data.children {
            let child = sections.get(child).ok_or(())?;
            completed &= child.state == SectionState::Completed;
        }

        if completed {
            let data = sections.get_mut(&self.id).unwrap();
            data.state = SectionState::Completed;
        }

        Ok(())
    }
}

#[pinned_drop]
impl PinnedDrop for Section {
    fn drop(self: Pin<&mut Self>) {
        if let Err(()) = self.check_completed() {
            if std::thread::panicking() {
                panic!("unexpected error during checking section completeness.");
            } else {
                eprintln!("warning: unexpected error during checking section completeness.");
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Sections {
    inner: Rc<RefCell<HashMap<&'static SectionId, SectionData>>>,
}

#[allow(clippy::new_without_default)]
impl Sections {
    pub(crate) fn new() -> Self {
        let mut inner = HashMap::new();
        inner.insert(
            &SectionId::Root,
            SectionData {
                state: SectionState::Found,
                children: vec![],
            },
        );
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub(crate) fn root(&self) -> Section {
        Section {
            sections: self.clone(),
            id: &SectionId::Root,
            encounted: false,
            _marker: PhantomPinned,
        }
    }

    pub(crate) fn completed(&self) -> bool {
        let sections = self.inner.borrow();
        let root = &sections[&SectionId::Root];
        root.state == SectionState::Completed
    }
}
