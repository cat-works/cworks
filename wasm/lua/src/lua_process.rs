use kernel::Process;

pub struct LuaProcess {
    lua_thread: mlua::Thread,
}

impl LuaProcess {
    #[must_use]
    pub fn new(lua_thread: mlua::Thread) -> Self {
        LuaProcess { lua_thread }
    }
}

impl Process for LuaProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult {
        let syscall_json = serde_json::to_string(data).expect("Failed to serialize SyscallData");

        let res: String = self
            .lua_thread
            .resume(syscall_json)
            .expect("Failed to resume Lua thread");

        let res_data: kernel::PollResult =
            serde_json::from_str(&res).expect("Failed to deserialize PollResult from Lua thread");

        res_data
    }
}
