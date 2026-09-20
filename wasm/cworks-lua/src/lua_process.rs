use kernel::obj_tree::{FSObjRef, Object};
use kernel::{Process, ProcessClientExt};
use kernel::{RustProcess, RustProcessCore};
use mlua::UserData;
use mlua::prelude::*;

#[derive(UserData)]
struct LuaFSObj(FSObjRef);

#[mlua::userdata_impl]
impl LuaFSObj {
    fn get_obj(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        let obj = self
            .0
            .get_obj(name)
            .map_err(|_| mlua::Error::external("get_obj failed"))?;
        Ok(LuaFSObj(obj))
    }

    fn list(&self) -> Result<Vec<String>, mlua::Error> {
        let list = self
            .0
            .list()
            .map_err(|_| mlua::Error::external("list failed"))?;
        Ok(list)
    }

    fn add_child(&self, name: &str, obj: &LuaAnyUserData) -> Result<(), mlua::Error> {
        self.0
            .add_child(name, &obj.borrow::<LuaFSObj>()?.0)
            .expect("add_child failed");
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_type(&self) -> Result<String, mlua::Error> {
        let obj = self.0.borrow();
        let type_str = match **obj {
            Object::CompoundFSObj { .. } => "CompoundFSObj",
            Object::Func { .. } => "Func",
            Object::String(_) => "String",
            Object::Boolean(_) => "Boolean",
            Object::Bytes(_) => "Bytes",
            Object::Double(_) => "Double",
            Object::Float(_) => "Float",
            Object::Int(_) => "Int",
            Object::Null => "Null",
        };
        Ok(type_str.to_string())
    }
}

#[derive(UserData)]
struct LuaSession(RustProcessCore);

#[mlua::userdata_impl]
impl LuaSession {
    async fn wait_for_event(&mut self) -> Result<(), mlua::Error> {
        self.0.wait_for_event().await.expect("a");
        Ok(())
    }

    async fn sleep(&mut self, sleep_duration: f32) -> Result<(), mlua::Error> {
        self.0.sleep(sleep_duration).await;
        Ok(())
    }

    async fn get_pid(&mut self) -> Result<u64, mlua::Error> {
        let pid = self.0.get_pid().await.expect("get_pid failed");
        Ok(pid)
    }

    async fn fs_root(&mut self) -> Result<LuaFSObj, mlua::Error> {
        let root = self.0.fs_root().await.expect("fs_root failed");
        Ok(LuaFSObj(root))
    }
}

pub fn new_lua_process(func: mlua::Function) -> Result<impl Process, mlua::Error> {
    let proc = RustProcess::new(
        &move |rpc: RustProcessCore, ()| {
            let value = func.clone();
            async move {
                let session = LuaSession(rpc);
                match value.call_async::<()>(session).await {
                    Ok(()) => log::info!("Lua process completed successfully"),
                    Err(e) => log::error!("Lua process failed: {e}"),
                }
            }
        },
        (),
    );

    Ok(proc)
}
