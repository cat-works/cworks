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

struct LuaSession {
    core: RustProcessCore,
}
impl UserData for LuaSession {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut("wait_for_event", |_, mut this, ()| async move {
            this.core.wait_for_event().await.expect("a");
            Ok(())
        });
        methods.add_async_method_mut("sleep", |_, mut this, sleep_duration: f32| async move {
            this.core.sleep(sleep_duration).await;
            Ok(())
        });
        methods.add_async_method_mut("get_pid", |_, mut this, ()| async move {
            let pid = this.core.get_pid().await.expect("get_pid failed");
            Ok(pid)
        });
        methods.add_async_method_mut("fs_root", |_, mut this, ()| async move {
            let root = this.core.fs_root().await.expect("fs_root failed");
            Ok(LuaFSObj(root))
        });
    }
}

pub fn new_lua_process(func: mlua::Function) -> Result<impl Process, mlua::Error> {
    let proc = RustProcess::new(
        &move |rpc: RustProcessCore, ()| {
            let value = func.clone();
            async move {
                let session = LuaSession { core: rpc };
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
