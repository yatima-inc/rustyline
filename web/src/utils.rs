#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = "log")]
    pub fn js_log(s: &str);
}

/// Logs to js console when compiled with debug
#[cfg(target_arch = "wasm32")]
#[cfg(debug_assertions)]
pub fn debug(s: &str) {
    js_log(&["[DEBUG]", s].join(" "));
}

#[cfg(not(debug_assertions))]
pub fn debug(_: &str) {
    // ignore
}

/// Logs to js console
#[cfg(target_arch = "wasm32")]
pub fn log(s: &str) {
    js_log(&["[INFO]", s].join(" "));
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ($crate::utils::log(&format!($($arg)*)));
}

// Not WASM
/// Logs when compiled with debug
#[cfg(debug_assertions)]
#[cfg(not(target_arch = "wasm32"))]
pub fn debug(s: &str) {
    io::stdout()
        .write_all(["[DEBUG]", s, "\n"].join(" ").as_bytes())
        .unwrap();
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::utils::debug(&format!($($arg)*)));
}

/// Logs info to std out
#[cfg(not(target_arch = "wasm32"))]
pub fn log(s: &str) {
    io::stdout()
        .write_all(["[INFO]", s, "\n"].join(" ").as_bytes())
        .unwrap();
}
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
