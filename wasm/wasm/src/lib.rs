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

pub mod buf_encoding;
pub mod ffi_luaenv;
pub mod ffi_luathread;
pub mod ffi_session;
pub mod js_callback;

use std::ffi::{CStr, CString, c_char};

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    log::info!("CWorks module initialized.");
}

/// Demangle a GHS-compiler mangled symbol. Returns a leaked C string
/// (same convention as the yield shim; ccall "string" copies it to JS).
#[unsafe(no_mangle)]
pub fn __ffi_demangle(x: *const c_char) -> *mut c_char {
    if x.is_null() {
        return std::ptr::null_mut();
    }
    let input: String = unsafe { CStr::from_ptr(x) }
        .to_string_lossy()
        .into_owned();
    let out = ghs_demangle::demangle(input).to_string();
    match CString::new(out) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
