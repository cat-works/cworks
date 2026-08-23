//! FFI shim over [`cworks_lua::LuaEnv`]. Pointer/CString marshaling only —
//! all Lua logic lives in the wrapper crate.

use std::ffi::{CStr, c_char};

use cworks_lua::{LuaEnv, LuaThread};

#[unsafe(no_mangle)]
pub fn __ffi_lufenv_new() -> *const LuaEnv {
    Box::into_raw(Box::new(LuaEnv::new())) as *const LuaEnv
}

#[unsafe(no_mangle)]
pub fn __ffi_luaenv_del(env: *const LuaEnv) {
    if env.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(env as *mut LuaEnv);
    }
}

#[unsafe(no_mangle)]
pub fn __ffi_luaenv_run(env: *const LuaEnv, code: *mut c_char) {
    if env.is_null() || code.is_null() {
        return;
    }
    let env = unsafe { &*env };

    let code: String = unsafe { CStr::from_ptr(code) }
        .to_string_lossy()
        .into_owned();

    env.run_code(code);
}

#[unsafe(no_mangle)]
pub fn __ffi_luaenv_thread(
    env: *const LuaEnv,
    name: *mut c_char,
    code: *mut c_char,
) -> *mut LuaThread {
    if env.is_null() || name.is_null() || code.is_null() {
        return std::ptr::null_mut();
    }
    let env = unsafe { &*env };

    let name: String = unsafe { CStr::from_ptr(name) }
        .to_string_lossy()
        .into_owned();

    let code: String = unsafe { CStr::from_ptr(code) }
        .to_string_lossy()
        .into_owned();

    let lua_thread = env.thread(&name, &code);
    Box::into_raw(Box::new(lua_thread))
}
