use crate::test_case::TestCase;
use std::{cell::Cell, collections::hash_map::Entry, fmt, ptr::NonNull};

thread_local! {
    static SECTION: Cell<Option<NonNull<Section>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Section>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        SECTION.with(|tls| tls.set(self.0.take()));
    }
}

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
pub(crate) struct SectionData {
    pub(crate) state: SectionState,
    pub(crate) children: Vec<&'static SectionId>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum SectionState {
    Found,
    Completed,
}

pub struct Section {
    pub(crate) test_case: TestCase,
    pub(crate) id: &'static SectionId,
    pub(crate) encounted: bool,
}

impl Section {
    pub(crate) fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let section_ptr = SECTION.with(|tls| tls.take());
        let _reset = SetOnDrop(section_ptr);
        let mut section_ptr = section_ptr.expect("section is not set on the current thread");
        unsafe { f(section_ptr.as_mut()) }
    }

    #[doc(hidden)] // private API.
    pub fn scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let prev = SECTION.with(|tls| tls.replace(Some(NonNull::from(self))));
        let _reset = SetOnDrop(prev);
        f()
    }

    pub(crate) fn new_section(&mut self, id: &'static SectionId) -> Option<Section> {
        let mut sections = self.test_case.sections.borrow_mut();
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
                test_case: self.test_case.clone(),
                id,
                encounted: false,
            })
        } else {
            None
        }
    }

    fn check_completed(&mut self) -> Result<(), ()> {
        let mut sections = self.test_case.sections.try_borrow_mut().map_err(drop)?;

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

impl Drop for Section {
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
