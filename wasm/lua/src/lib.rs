pub mod buf_encoding;
pub mod ffi_session;
pub mod js_callback;
pub mod luaenv;
pub mod luathread;

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    log::info!("CWorks module initialized.");
}

/// Demangle a GHS-compiler mangled symbol. Returns a leaked C string
/// (same convention as `__ffi_lua_thread_yield`; ccall "string" copies it to JS).
#[unsafe(no_mangle)]
pub fn __ffi_demangle(x: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    if x.is_null() {
        return std::ptr::null_mut();
    }
    let input: String = unsafe { std::ffi::CStr::from_ptr(x) }
        .to_string_lossy()
        .into_owned();
    let out = ghs_demangle::demangle(input).to_string();
    match std::ffi::CString::new(out) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
