/*!
Catch inspired testing framework for Rust.
!*/

#[doc(hidden)]
pub mod tls;

use futures::future::Future;
use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::fmt;
use std::rc::Rc;

pub fn test_case<'a, F>(f: F)
where
    F: Fn() + 'a,
{
    let sections = Sections::new();
    while !sections.completed() {
        let mut section = sections.root();
        let _guard = crate::tls::set(&mut section);
        f();
    }
}

pub async fn test_case_async<'a, F, Fut>(f: F)
where
    F: Fn() -> Fut + 'a,
    Fut: Future<Output = ()> + 'a,
{
    crate::tls::with_tls(async move {
        let sections = Sections::new();
        while !sections.completed() {
            let mut section = sections.root();
            let _guard = crate::tls::set(&mut section);
            f().await;
        }
    })
    .await
}

#[macro_export]
macro_rules! section {
    ($name:expr, $body:block) => {{
        static SECTION: $crate::SectionId = $crate::SectionId::SubSection {
            name: $name,
            file: file!(),
            line: line!(),
            column: column!(),
        };
        if let Some(mut section) = $crate::tls::with(|section| section.new_section(SECTION)) {
            let _guard = $crate::tls::set(&mut section);
            $body
        }
    }};
}

#[doc(hidden)]
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

/// A container that holds all section data in a test case.
#[derive(Clone)]
struct Sections {
    inner: Rc<RefCell<HashMap<SectionId, SectionData>>>,
}

#[allow(clippy::new_without_default)]
impl Sections {
    fn new() -> Self {
        let mut inner = HashMap::new();
        inner.insert(
            SectionId::Root,
            SectionData {
                state: SectionState::Found,
                children: vec![],
            },
        );
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    fn root(&self) -> Section {
        Section {
            sections: self.clone(),
            id: SectionId::Root,
            encounted: false,
        }
    }

    fn completed(&self) -> bool {
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

#[doc(hidden)]
pub struct Section {
    sections: Sections,
    id: SectionId,
    encounted: bool,
}

impl Section {
    pub fn new_section(&mut self, id: SectionId) -> Option<Section> {
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
                sections: self.sections.clone(),
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
