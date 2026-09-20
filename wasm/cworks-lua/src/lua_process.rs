use kernel::obj_tree::{FSObjRef, Object};
use kernel::{Process, ProcessClientExt};
use kernel::{RustProcess, RustProcessCore};
use mlua::UserData;
use mlua::prelude::*;

#[derive(UserData)]
struct LuaFSObj(FSObjRef);

impl LuaFSObj {
    fn add_child(&self, name: &str, obj: &FSObjRef) -> Result<FSObjRef, mlua::Error> {
        if self
            .0
            .has_child(name)
            .map_err(|_| mlua::Error::external("has_child failed"))?
        {
            return Err(mlua::Error::external(format!(
                "child with name '{name}' already exists"
            )));
        }
        self.0.add_child(name, &obj).expect("add_child failed");
        Ok(obj.clone())
    }
}

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

    #[allow(clippy::unnecessary_wraps)]
    #[lua(name = "type")]
    fn type_(&self) -> Result<String, mlua::Error> {
        let obj = self.0.borrow();
        let type_str = match **obj {
            Object::Int(_) => "Int",
            Object::String(_) => "String",
            Object::Boolean(_) => "Boolean",
            Object::Float(_) => "Float",
            Object::Double(_) => "Double",
            Object::Bytes(_) => "Bytes",
            Object::Null => "Null",
            Object::CompoundFSObj { .. } => "CompoundFSObj",
            Object::Func { .. } => "Func",
        };
        Ok(type_str.to_string())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn new_int(value: i128) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Int(value));
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_string(value: String) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::String(value));
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_boolean(value: bool) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Boolean(value));
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_float(value: f32) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Float(value));
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_double(value: f64) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Double(value));
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_bytes(value: Vec<u8>) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Bytes(value));
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_null() -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Null);
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_compound() -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::CompoundFSObj {
            parent: None,
            children: std::collections::HashMap::new(),
        });
        Ok(LuaFSObj(obj))
    }
    #[allow(clippy::unnecessary_wraps)]
    fn new_func() -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Func { callee_pid: vec![] });
        Ok(LuaFSObj(obj))
    }

    fn create_int(&self, name: &str, value: i128) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Int(value));

        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_string(&self, name: &str, value: String) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::String(value));
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_boolean(&self, name: &str, value: bool) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Boolean(value));
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_float(&self, name: &str, value: f32) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Float(value));
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_double(&self, name: &str, value: f64) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Double(value));
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_bytes(&self, name: &str, value: Vec<u8>) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Bytes(value));
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_null(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Null);
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_compound(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::CompoundFSObj {
            parent: Some(self.0.clone()),
            children: std::collections::HashMap::new(),
        });
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn create_func(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        let obj = FSObjRef::from(Object::Func { callee_pid: vec![] });
        self.add_child(name, &obj).map(LuaFSObj)
    }

    fn ensure_int(&self, name: &str, value: i128) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_int(name, value),
        }
    }
    fn ensure_string(&self, name: &str, value: String) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_string(name, value),
        }
    }
    fn ensure_boolean(&self, name: &str, value: bool) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_boolean(name, value),
        }
    }
    fn ensure_float(&self, name: &str, value: f32) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_float(name, value),
        }
    }
    fn ensure_double(&self, name: &str, value: f64) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_double(name, value),
        }
    }
    fn ensure_bytes(&self, name: &str, value: Vec<u8>) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_bytes(name, value),
        }
    }
    fn ensure_null(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_null(name),
        }
    }
    fn ensure_compound(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_compound(name),
        }
    }
    fn ensure_func(&self, name: &str) -> Result<LuaFSObj, mlua::Error> {
        match self.0.get_obj(name) {
            Ok(obj) => Ok(LuaFSObj(obj)),
            Err(_) => self.create_func(name),
        }
    }

    fn string(&self) -> Result<String, mlua::Error> {
        let obj = self.0.borrow();
        match &**obj {
            Object::String(s) => Ok(s.clone()),
            _ => Err(mlua::Error::external("not a string object")),
        }
    }

    fn int(&self) -> Result<i128, mlua::Error> {
        let obj = self.0.borrow();
        match &**obj {
            Object::Int(i) => Ok(*i),
            _ => Err(mlua::Error::external("not an int object")),
        }
    }

    fn boolean(&self) -> Result<bool, mlua::Error> {
        let obj = self.0.borrow();
        match &**obj {
            Object::Boolean(b) => Ok(*b),
            _ => Err(mlua::Error::external("not a boolean object")),
        }
    }

    fn float(&self) -> Result<f32, mlua::Error> {
        let obj = self.0.borrow();
        match &**obj {
            Object::Float(f) => Ok(*f),
            _ => Err(mlua::Error::external("not a float object")),
        }
    }

    fn double(&self) -> Result<f64, mlua::Error> {
        let obj = self.0.borrow();
        match &**obj {
            Object::Double(d) => Ok(*d),
            _ => Err(mlua::Error::external("not a double object")),
        }
    }

    fn bytes(&self) -> Result<Vec<u8>, mlua::Error> {
        let obj = self.0.borrow();
        match &**obj {
            Object::Bytes(b) => Ok(b.clone()),
            _ => Err(mlua::Error::external("not a bytes object")),
        }
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

    async fn subscribe(
        &mut self,
        obj: LuaAnyUserData,
        handler: mlua::Function,
    ) -> Result<(), mlua::Error> {
        let obj_ref = obj.borrow::<LuaFSObj>()?.0.clone();
        let data_handler = std::rc::Rc::new(Box::new(move |data: Option<FSObjRef>| {
            if let Some(obj) = data {
                let lua_obj = LuaFSObj(obj);
                handler
                    .call::<()>(lua_obj)
                    .map_err(|e| {
                        log::error!("Error in Lua handler: {e}");
                        mlua::Error::external("Lua handler failed")
                    })
                    .unwrap();
            }
            Ok(())
        })
            as Box<dyn Fn(Option<FSObjRef>) -> Result<(), kernel::SyscallError>>);

        self.0
            .subscribe(obj_ref, data_handler)
            .await
            .map_err(|_| mlua::Error::external("subscribe failed"))?;
        Ok(())
    }

    async fn unsubscribe(&mut self, obj: LuaAnyUserData) -> Result<(), mlua::Error> {
        let obj_ref = obj.borrow::<LuaFSObj>()?.0.clone();
        self.0
            .unsubscribe(obj_ref)
            .await
            .map_err(|_| mlua::Error::external("unsubscribe failed"))?;
        Ok(())
    }

    async fn publish(
        &mut self,
        obj: LuaAnyUserData,
        data: Option<LuaAnyUserData>,
    ) -> Result<(), mlua::Error> {
        let obj_ref = obj.borrow::<LuaFSObj>()?.0.clone();
        let data_ref = if let Some(data) = data {
            Some(data.borrow::<LuaFSObj>()?.0.clone())
        } else {
            None
        };
        self.0
            .publish(obj_ref, data_ref)
            .await
            .map_err(|_| mlua::Error::external("publish failed"))?;
        Ok(())
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
