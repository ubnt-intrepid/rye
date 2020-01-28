/*!
Catch inspired testing framework for Rust.
!*/

use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::fmt;

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

#[macro_export]
macro_rules! section_id {
    ($name:expr) => {
        $crate::SectionId::SubSection {
            name: $name,
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}

/// A container that holds all section data in a test case.
pub struct Sections {
    inner: RefCell<HashMap<SectionId, SectionData>>,
}

#[allow(clippy::new_without_default)]
impl Sections {
    pub fn new() -> Self {
        let mut inner = HashMap::new();
        inner.insert(
            SectionId::Root,
            SectionData {
                state: SectionState::Found,
                children: vec![],
            },
        );
        Self {
            inner: RefCell::new(inner),
        }
    }

    pub fn root(&self) -> Section<'_> {
        Section {
            sections: self,
            id: SectionId::Root,
            encounted: false,
        }
    }

    pub fn completed(&self) -> bool {
        let sections = self.inner.borrow();
        let root = &sections[&SectionId::Root];
        root.state == SectionState::Completed
    }
}

#[derive(Debug)]
struct SectionData {
    state: SectionState,
    children: Vec<SectionId>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SectionState {
    Found,
    Completed,
}

pub struct Section<'a> {
    sections: &'a Sections,
    id: SectionId,
    encounted: bool,
}

impl<'a> Section<'a> {
    pub fn new_section(&mut self, id: SectionId) -> Option<Section<'_>> {
        let mut sections = self.sections.inner.borrow_mut();
        let insert_child;
        let is_target;
        match sections.entry(id) {
            Entry::Occupied(entry) => {
                let data = entry.into_mut();
                match data.state {
                    SectionState::Found if !self.encounted => {
                        self.encounted = true;
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
                if self.encounted {
                    entry.insert(SectionData {
                        state: SectionState::Found,
                        children: vec![],
                    });
                    insert_child = true;
                    is_target = false;
                } else {
                    self.encounted = true;
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
            sections.get_mut(&self.id).unwrap().children.push(id);
        }

        if is_target {
            Some(Section {
                sections: &*self.sections,
                id,
                encounted: false,
            })
        } else {
            None
        }
    }

    fn check_completed(&mut self) -> Result<(), ()> {
        let mut sections = self.sections.inner.try_borrow_mut().map_err(drop)?;

        let mut completed = true;
        let data = sections.get(&self.id).ok_or(())?;
        for child in &data.children {
            let child = sections.get(&child).ok_or(())?;
            completed &= child.state == SectionState::Completed;
        }

        if completed {
            let data = sections.get_mut(&self.id).unwrap();
            data.state = SectionState::Completed;
        }

        Ok(())
    }
}

impl Drop for Section<'_> {
    fn drop(&mut self) {
        if let Err(()) = self.check_completed() {
            if std::thread::panicking() {
                panic!("unexpected error during checking section completeness.");
            } else {
                eprintln!("warning: unexpected error during checking section completeness.");
            }
        }
    }
}
