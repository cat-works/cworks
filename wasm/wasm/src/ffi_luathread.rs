//! FFI shim over [`cworks_lua::LuaThread`]. This is the wire-encoding
//! boundary: incoming C strings are NUL-decoded before entering the wrapper,
//! outgoing results are re-encoded. The error sentinel `\x01\x03` survives a
//! decode round-trip by construction and is matched by the TS wrapper.

use std::ffi::{CStr, CString, c_char};

use cworks_lua::{LuaThread, LuaThreadError};

use crate::buf_encoding::{decode, encode};

#[unsafe(no_mangle)]
pub fn __ffi_lua_thread_yield(thread: *mut LuaThread, data: *const c_char) -> *mut c_char {
    if thread.is_null() || data.is_null() {
        return std::ptr::null_mut();
    }

    let thread = unsafe { &mut *thread };

    let data: Vec<u8> = unsafe { CStr::from_ptr(data).to_bytes().to_vec() };
    let data = decode(&data);

    match thread.yield_process(data) {
        Ok(result) => {
            let result = encode(&result);
            let c_string = CString::new(result).unwrap();
            c_string.into_raw()
        }
        Err(e) => {
            match e {
                LuaThreadError::Mlua(err) => {
                    log::error!("MLua error in Lua thread, {err}");
                }
                LuaThreadError::InvalidData => {
                    log::error!("Invalid data received from Lua thread.");
                }
            }
            c"\x01\x03".as_ptr().cast_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub fn __ffi_lua_thread_del(thread: *mut LuaThread) {
    if thread.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(thread);
    }
}
