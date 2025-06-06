use std::ffi::c_char;

use mlua::{Lua, Thread};

use crate::luathread::LuaThread;

pub struct LuaEnv(pub Lua);

impl LuaEnv {
    pub fn new() -> Self {
        LuaEnv(Lua::new())
    }

    pub fn run_code(&self, str: String) {
        self.0.load(str).exec().unwrap()
    }
}

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

    let code: String = unsafe { std::ffi::CStr::from_ptr(code) }
        .to_string_lossy()
        .into_owned();

    env.run_code(code);
}

#[unsafe(no_mangle)]
pub fn __ffi_luaenv_thread(env: *mut LuaEnv, code: *mut c_char) -> *mut LuaThread {
    if env.is_null() || code.is_null() {
        return std::ptr::null_mut();
    }
    let env = unsafe { &*env };

    let code: String = unsafe { std::ffi::CStr::from_ptr(code) }
        .to_string_lossy()
        .into_owned();

    let mut wrapped_code = "".to_string();
    wrapped_code += "coroutine.create(function(arg)\n";
    wrapped_code += &code;
    wrapped_code += "\nend)";

    let thread: Thread = env
        .0
        .load(wrapped_code)
        .set_name("LuaProcessThread")
        .eval()
        .expect("Failed to create coroutine");

    let lua_thread = LuaThread::new(thread);
    let boxed_thread = Box::new(lua_thread);
    let ptr = Box::into_raw(boxed_thread);

    ptr as *mut LuaThread
}
