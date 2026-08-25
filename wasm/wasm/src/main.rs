//! CWorks artifact crate: the entire JS <-> Rust boundary.
//!
//! Every `#[unsafe(no_mangle)] pub extern "C"` export consumed from JS lives
//! here, together with the emscripten glue under `glue/`:
//!
//! - `glue/callback-pre.js` — JS-side callback registry (`Module._registerJsCallback`,
//!   `Module._invokeJsCallback`). Linked via `--pre-js`.
//! - `glue/cworks-lib.js` — provides the `cworks_js_callback` C function that
//!   `js_callback.rs` declares as `extern "C"`. Linked via `--js-library`.
//!
//! The Lua VM itself is wrapped by `cworks-lua`; this crate only marshals
//! data across the C boundary (pointers, CStrings, wire encoding).

pub mod ffi_luaenv;
pub mod ffi_luaprocess;
pub mod ffi_session;
pub mod js_callback;

use std::ffi::{CStr, CString, c_char};

fn main() {
    // This function is never called; it's just here to make sure the crate compiles.
    // The actual entry point is `__ffi_init`, which is called from JS.
}

/// Console output routed by log level (emscripten console API / native std).
/// Trace/Debug -> console.debug, Info -> console.log, Warn -> console.warn,
/// Error -> console.error so browser level filters work as-is.
mod console {
    use log::Level;

    #[cfg(target_os = "emscripten")]
    pub fn write(level: Level, msg: &str) {
        use std::ffi::{CString, c_char};

        unsafe extern "C" {
            fn emscripten_console_log(utf8_string: *const c_char);
            fn emscripten_console_warn(utf8_string: *const c_char);
            fn emscripten_console_error(utf8_string: *const c_char);
            fn cworks_console_debug(utf8_string: *const c_char);
        }

        let c = CString::new(msg).unwrap();
        unsafe {
            match level {
                Level::Error => emscripten_console_error(c.as_ptr()),
                Level::Warn => emscripten_console_warn(c.as_ptr()),
                Level::Info => emscripten_console_log(c.as_ptr()),
                Level::Debug | Level::Trace => cworks_console_debug(c.as_ptr()),
            }
        }
    }

    #[cfg(not(target_os = "emscripten"))]
    pub fn write(level: Level, msg: &str) {
        let msg = msg.replace('\n', "\\n");
        match level {
            Level::Error => eprintln!("{msg}"),
            Level::Warn => eprintln!("{msg}"),
            _ => println!("{msg}"),
        }
    }
}

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        // Single-line records keep one console entry per log call.
        let msg = format!("[{}] {}", record.target(), record.args()).replace('\n', "\\n");
        console::write(record.level(), &msg);
    }

    fn flush(&self) {}
}

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_init() {
    let _ = log::set_boxed_logger(Box::new(ConsoleLogger));
    log::set_max_level(log::LevelFilter::Trace);

    log::info!("CWorks module initialized.");
}

/// Demangle a GHS-compiler mangled symbol. Returns a leaked C string
/// (same convention as the yield shim; ccall "string" copies it to JS).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ffi_demangle(x: *const c_char) -> *mut c_char {
    if x.is_null() {
        return std::ptr::null_mut();
    }
    let input: String = unsafe { CStr::from_ptr(x) }.to_string_lossy().into_owned();
    let out = ghs_demangle::demangle(input).to_string();
    match CString::new(out) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
