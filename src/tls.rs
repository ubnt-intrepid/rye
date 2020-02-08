use crate::section::Section;
use std::{cell::Cell, ptr::NonNull};

thread_local! {
    static SECTION: Cell<Option<NonNull<Section>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Section>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        SECTION.with(|tls| tls.set(self.0.take()));
    }
}

pub(crate) fn with<F, R>(f: F) -> R
where
    F: FnOnce(&mut Section) -> R,
{
    let section_ptr = SECTION.with(|tls| tls.take());
    let _reset = SetOnDrop(section_ptr);
    let mut section_ptr = section_ptr.expect("section is not set on the current thread");
    unsafe { f(section_ptr.as_mut()) }
}

pub(crate) fn set<F, R>(section: &mut Section, f: F) -> R
where
    F: FnOnce() -> R,
{
    let prev = SECTION.with(|tls| tls.replace(Some(NonNull::from(section))));
    let _reset = SetOnDrop(prev);
    f()
}
