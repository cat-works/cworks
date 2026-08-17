use std::ffi::c_char;

use mlua::Thread;

use crate::buf_encoding::{decode, encode};

enum LuaThreadError {
    MLuaError(mlua::Error),
    InvalidData,
}

pub struct LuaThread {
    thread: Thread,
}

impl LuaThread {
    #[must_use]
    pub fn new(thread: Thread) -> Self {
        LuaThread { thread }
    }

    fn yield_process(&mut self, data: Vec<u8>) -> Result<Vec<u8>, LuaThreadError> {
        let lua_data = mlua::String::wrap(data);

        let result: mlua::Result<mlua::Value> = self.thread.resume(lua_data);

        result.map_err(LuaThreadError::MLuaError).and_then(|x| {
            if let mlua::Value::String(s) = x {
                Ok(s.as_bytes().to_vec())
            } else {
                log::error!("Expected a string from Lua thread, got: {x:?}");
                Err(LuaThreadError::InvalidData)
            }
        })
    }
}

#[unsafe(no_mangle)]
pub fn __ffi_lua_thread_yield(thread: *mut LuaThread, data: *const c_char) -> *mut c_char {
    if thread.is_null() || data.is_null() {
        return std::ptr::null_mut();
    }

    let thread = unsafe { &mut *thread };

    let data: Vec<u8> = unsafe { std::ffi::CStr::from_ptr(data).to_bytes().to_vec() };
    let data = decode(&data);

    match thread.yield_process(data) {
        Ok(result) => {
            let result = encode(&result);
            let c_string = std::ffi::CString::new(result).unwrap();
            c_string.into_raw() // Return a raw pointer to the C string
        }
        Err(e) => {
            match e {
                LuaThreadError::MLuaError(err) => {
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
