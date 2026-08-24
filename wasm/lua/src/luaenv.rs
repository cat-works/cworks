use mlua::Lua;

use crate::luathread::LuaThread;

pub struct LuaEnv(Lua);

impl LuaEnv {
    pub fn new() -> Self {
        LuaEnv(Lua::new())
    }

    pub fn run_code(&self, str: String) {
        self.0.load(str).exec().unwrap()
    }

    /// Create a coroutine-wrapped thread from `code`, resumable via
    /// [`LuaThread::yield_process`].
    pub fn thread(&self, name: &str, code: &str) -> LuaThread {
        let mut wrapped_code = "".to_string();
        wrapped_code += "coroutine.create(function(arg)\n";
        wrapped_code += code;
        wrapped_code += "\nend)";

        let thread = self
            .0
            .load(wrapped_code)
            .set_name(name)
            .eval()
            .expect("Failed to create coroutine");

        LuaThread::new(thread)
    }

    pub fn get_lua(&self) -> &Lua {
        &self.0
    }
}
