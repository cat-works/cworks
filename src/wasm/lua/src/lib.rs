mod handle_casher;

use std::rc::Rc;

use kernel::{PollResult, Process, SyscallData, SyscallError};
use log::debug;
use mlua::{Lua, Thread};

pub struct LuaEnv(pub Lua);
impl LuaEnv {
    pub fn new() -> Self {
        LuaEnv(Lua::new())
    }
}

pub trait RcLuaEnv {
    fn run(&self, code: String) -> LuaProcess;
}

impl RcLuaEnv for Rc<LuaEnv> {
    fn run(&self, code: String) -> LuaProcess {
        LuaProcess::new(self.clone(), code)
    }
}

pub struct LuaProcess {
    lua: Rc<LuaEnv>,
    thread: Thread,

    handle_casher: handle_casher::HandleCasher,
}

impl LuaProcess {
    pub fn new(lua: Rc<LuaEnv>, code: String) -> Self {
        let mut wrapped_code = "".to_string();
        wrapped_code += "coroutine.create(function(arg)\n";
        wrapped_code += &code;
        wrapped_code += "\nend)";

        let thread: Thread = lua
            .0
            .load(wrapped_code)
            .set_name("LuaProcessThread")
            .eval()
            .expect("Failed to create coroutine");

        LuaProcess {
            lua,
            thread,
            handle_casher: handle_casher::HandleCasher::default(),
        }
    }

    fn syscall_data_to_vec_u8(&mut self, value: SyscallData) -> Vec<u8> {
        match value {
            SyscallData::None => vec![0x00],
            SyscallData::Fail(SyscallError::AlreadyExists) => vec![0x01],
            SyscallData::Fail(SyscallError::NoSuchEntry) => vec![0x02],
            SyscallData::Fail(SyscallError::NotImplemented) => vec![0x03],
            SyscallData::Fail(SyscallError::ResourceIsBusy) => vec![0x04],
            SyscallData::Fail(SyscallError::UnknownHandle) => vec![0x05],
            SyscallData::Fail(SyscallError::UnreachableEntry) => vec![0x06],
            SyscallData::Handle(handle) => {
                self.handle_casher.register_handle(handle.clone());

                let mut a = vec![0x07];
                a.extend(handle.id.to_be_bytes());
                a
            }
            SyscallData::Connection { client, server } => {
                self.handle_casher.register_handle(client.clone());
                self.handle_casher.register_handle(server.clone());

                let mut a = vec![0x08];
                a.extend(client.id.to_be_bytes());
                a.extend(server.id.to_be_bytes());

                a
            }
            SyscallData::ReceivingData { focus, data } => {
                self.handle_casher.register_handle(focus.clone());

                let mut a = vec![0x09];
                a.extend(focus.id.to_be_bytes());
                a.extend(data.into_bytes());
                a
            }
        }
    }

    fn vec_u8_to_poll_result(&mut self, value: Vec<u8>) -> PollResult<i64> {
        match value[0] {
            0x00 => PollResult::Pending,
            0x01 => {
                let return_value = i64::from_be_bytes(
                    value[1..9]
                        .try_into()
                        .expect("Failed to convert bytes to i64"),
                );
                PollResult::Done(return_value)
            }
            0x02 => {
                // 1..5: f32 (duration)
                let duration = f32::from_be_bytes(
                    value[1..5]
                        .try_into()
                        .expect("Failed to convert bytes to f32"),
                );
                PollResult::Syscall(kernel::Syscall::Sleep(duration))
            }
            0x03 => {
                let ipc_name = String::from_utf8(value[1..].to_vec())
                    .expect("Failed to convert bytes to String");
                PollResult::Syscall(kernel::Syscall::IpcCreate(ipc_name))
            }
            0x04 => {
                let ipc_name = String::from_utf8(value[1..].to_vec())
                    .expect("Failed to convert bytes to String");
                PollResult::Syscall(kernel::Syscall::IpcConnect(ipc_name))
            }
            0x05 => {
                // 1..17: u128 (handle ID)
                // 17..: String (data)

                let handle_id = u128::from_be_bytes(
                    value[1..17]
                        .try_into()
                        .expect("Failed to convert bytes to u128"),
                );
                let handle = self.handle_casher.get_handle(handle_id).unwrap();

                let data = String::from_utf8(value[17..].to_vec())
                    .expect("Failed to convert bytes to String");

                PollResult::Syscall(kernel::Syscall::Send(handle, data))
            }
            _ => panic!("Unknown syscall data type"),
        }
    }
}

impl Process for LuaProcess {
    fn poll(&mut self, data: &SyscallData) -> kernel::PollResult<i64> {
        let lua_data = self.syscall_data_to_vec_u8(data.clone());
        let lua_data = mlua::Value::String(
            self.lua
                .0
                .create_string(&lua_data)
                .expect("Failed to create Lua string"),
        );

        let result: mlua::Result<mlua::Value> = self.thread.resume(lua_data.clone());
        // debug!("Lua thread resumed with data: {:?}", lua_data);
        // debug!("Lua thread result: {:?}", result);

        match result {
            Ok(mlua::Value::Integer(i)) => {
                debug!("Lua returned integer: {}", i);
                PollResult::Done(i)
            }
            Ok(mlua::Value::String(s)) => {
                let bytes = s.as_bytes().to_vec();
                let syscall_data = self.vec_u8_to_poll_result(bytes);
                syscall_data
            }
            _ => panic!("Unexpected Lua return value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use kernel::Kernel;

    use super::*;

    #[test]
    fn lua_works() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let env = Rc::new(LuaEnv::new());
        let mut kernel = Kernel::default();

        let thread = env.run(
            r#"
        --- Dumps a string with hexadecimal representation to the console
        function dump_str(s)
            local dump = ""
            for i = 1, #s do
                dump = dump .. string.format("%02X ", string.byte(s, i))
            end
            return dump
        end

        print("Arg: " .. dump_str(arg))

        coroutine.yield("\x00")
        coroutine.yield("\x00")
        coroutine.yield("\x00")

        local yieldarg = coroutine.yield("\x04system/file-system")
        print("Yielded with: " .. dump_str(yieldarg))

        coroutine.yield("\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02Stat?/")
        local yieldarg = coroutine.yield("\x00")
        local yieldarg = coroutine.yield("\x00")
        print("Yielded with: " .. dump_str(yieldarg))

        return 987
        "#
            .to_string(),
        );
        kernel.register_process(Box::new(thread));

        kernel.start();
    }
}
