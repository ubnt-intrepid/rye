#![allow(missing_docs)]

use maybe_unwind::capture_panic_info;
use std::panic;
use std::sync::Once;

static INSTALL_GLOBALS: Once = Once::new();

pub fn install_globals() {
    INSTALL_GLOBALS.call_once(|| {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !capture_panic_info(info) {
                prev_hook(info);
            }
        }));
    });
}
