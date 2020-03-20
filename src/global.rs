use crate::executor::Context;
use std::{panic, sync::Once};

pub(crate) fn install() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        install_panic_hook();
    });
}

fn install_panic_hook() {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if Context::is_set() {
            let _ = Context::try_with(|ctx| ctx.capture_panic_info(info));
            return;
        }
        prev_hook(info);
    }));
}
