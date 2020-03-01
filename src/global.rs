use std::{
    panic::{self, PanicInfo},
    sync::Once,
};

pub(crate) fn panic_hook(info: &PanicInfo) {
    maybe_unwind::capture_panic_info(info);
}

pub(crate) fn install() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        panic::set_hook(Box::new(panic_hook));
    });
}
