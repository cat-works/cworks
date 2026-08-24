use std::ffi::{CStr, c_char};

use mlua::Lua;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ffi_lufenv_new() -> *mut Lua {
    Box::into_raw(Box::new(Lua::new()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ffi_luaenv_del(env: *mut Lua) {
    if env.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(env);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ffi_luaenv_run(env: *const Lua, code: *mut c_char) {
    if env.is_null() || code.is_null() {
        return;
    }
    let env = unsafe { &*env };

    let code: String = unsafe { CStr::from_ptr(code) }
        .to_string_lossy()
        .into_owned();

    env.load(&code).exec().expect("Failed to execute Lua code");
}
