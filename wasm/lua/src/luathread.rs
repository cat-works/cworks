use mlua::Thread;

pub struct LuaThread {
    thread: Thread,
}

#[derive(Debug)]
pub enum LuaThreadError {
    /// mlua error, stringified to keep mlua types out of the public API.
    Mlua(String),
    InvalidData,
}

impl LuaThread {
    #[must_use]
    pub fn new(thread: Thread) -> Self {
        LuaThread { thread }
    }

    pub fn yield_process(&mut self, data: Vec<u8>) -> Result<Vec<u8>, LuaThreadError> {
        let lua_data = mlua::String::wrap(data);

        let result: mlua::Result<mlua::Value> = self.thread.resume(lua_data);

        result
            .map_err(|e| LuaThreadError::Mlua(e.to_string()))
            .and_then(|x| {
                if let mlua::Value::String(s) = x {
                    Ok(s.as_bytes().to_vec())
                } else {
                    log::error!("Expected a string from Lua thread, got: {x:?}");
                    Err(LuaThreadError::InvalidData)
                }
            })
    }
}
