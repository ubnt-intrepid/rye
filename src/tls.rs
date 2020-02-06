use crate::section::Section;
use std::{cell::Cell, marker::PhantomData};

thread_local! {
    static CURRENT_SECTION: Cell<Option<Box<Section>>> = Cell::new(None);
}

pub struct Guard<'a> {
    old_section: Option<Box<Section>>,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        CURRENT_SECTION.with(|tls| {
            tls.set(self.old_section.take());
        });
    }
}

impl<'a> Guard<'a> {
    #[inline(never)]
    pub fn set(section: Option<Box<Section>>) -> Self {
        let old_section = CURRENT_SECTION.with(|tls| tls.replace(section));
        Self {
            old_section,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn section_mut(&mut self) -> Option<&mut Section> {
        self.old_section.as_deref_mut()
    }
}

pub(crate) fn with<F, R>(f: F) -> R
where
    F: FnOnce(&mut Section) -> R,
{
    let mut guard = Guard::set(None);
    let section = guard.section_mut().expect("current section is not set");
    f(section)
}

#[cfg(feature = "futures")]
pub(crate) fn take() -> Option<Box<Section>> {
    CURRENT_SECTION.with(|tls| tls.take())
}
