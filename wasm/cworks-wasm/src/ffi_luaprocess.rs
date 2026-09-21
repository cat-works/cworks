use std::ffi::{CStr, c_char};

use cworks_lua::new_lua_process;
use mlua::Lua;

use crate::ffi_session::SESSION;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ffi_lua_process_new(
    env: *mut Lua,
    name: *mut c_char,
    code: *mut c_char,
) -> u64 {
    if env.is_null() || name.is_null() || code.is_null() {
        return u64::MAX;
    }
    let env = unsafe { &*env };

    let name: String = unsafe { CStr::from_ptr(name) }
        .to_string_lossy()
        .into_owned();

    let code: String = unsafe { CStr::from_ptr(code) }
        .to_string_lossy()
        .into_owned();

    let func = env
        .load(code)
        .set_name(&name)
        .into_function()
        .expect("Failed to load Lua function");

    let lua_process = Box::new(new_lua_process(func).expect("Failed to create Lua process"));

    let pid = SESSION.with(|session| {
        let mut session = session.borrow_mut();
        session
            .as_mut()
            .expect("Session not initialized")
            .register_process(lua_process)
    });

    pid as u64
}
