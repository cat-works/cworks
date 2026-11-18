use std::cell::RefCell;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use kernel::obj_tree::{FSObjRef, Object};
use kernel::{Kernel, PollResult, Process, SyscallData};
use wasm_bindgen::prelude::*;

thread_local! {
    static SESSION: RefCell<Option<Kernel>> = const { RefCell::new(None) };
}

fn with_session<R>(f: impl FnOnce(&mut Kernel) -> R) -> R {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        f(s.as_mut().expect("cworks session is not initialized"))
    })
}

#[wasm_bindgen(js_name = demangleStr)]
pub fn demangle_str(x: &str) -> String {
    ghs_demangle::demangle(x.to_string()).to_string()
}

#[derive(Clone, Default)]
struct PendingRequest(Rc<RefCell<Option<PollResult>>>);

impl PendingRequest {
    fn set(&self, request: PollResult) {
        *self.0.borrow_mut() = Some(request);
    }

    fn clear(&self) {
        *self.0.borrow_mut() = None;
    }

    fn take(&self) -> PollResult {
        self.0.borrow_mut().take().unwrap_or(PollResult::Pending)
    }
}

#[wasm_bindgen]
pub struct Session;

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Session {
        SESSION.with(|s| *s.borrow_mut() = Some(Kernel::default()));
        Session
    }

    pub fn step(&self) {
        with_session(Kernel::step);
    }

    #[wasm_bindgen(js_name = addProcess)]
    pub fn add_process(&self, callback: js_sys::Function) -> JsProcessHandle {
        let pending = PendingRequest::default();
        with_session(|k| {
            k.register_process(Box::new(JsProcess {
                callback,
                pending: pending.clone(),
            }))
        });
        JsProcessHandle(AssertUnwindSafe(pending))
    }
}

#[wasm_bindgen]
pub struct JsProcessHandle(AssertUnwindSafe<PendingRequest>);

#[wasm_bindgen]
impl JsProcessHandle {
    pub fn done(&self) {
        self.0.0.set(PollResult::Done);
    }

    pub fn root(&self) {
        self.0.0.set(PollResult::Root);
    }

    #[wasm_bindgen(js_name = waitForEvent)]
    pub fn wait_for_event(&self) {
        self.0.0.set(PollResult::WaitForEvent);
    }

    pub fn sleep(&self, seconds: f32) {
        self.0.0.set(PollResult::Sleep(seconds));
    }

    #[wasm_bindgen(js_name = getPid)]
    pub fn get_pid(&self) {
        self.0.0.set(PollResult::GetPid);
    }
}

#[wasm_bindgen]
pub struct FSObj(AssertUnwindSafe<FSObjRef>);

impl FSObj {
    fn add_child(&self, name: &str, obj: FSObjRef) -> Result<FSObj, JsError> {
        self.0
            .0
            .add_child(name, &obj)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(FSObj(AssertUnwindSafe(obj)))
    }
}

#[wasm_bindgen]
impl FSObj {
    pub fn id(&self) -> u32 {
        self.0.0.as_ptr() as u32
    }

    #[wasm_bindgen(js_name = getObj)]
    pub fn get_obj(&self, name: &str) -> Result<FSObj, JsError> {
        self.0
            .0
            .get_obj(name)
            .map(|o| FSObj(AssertUnwindSafe(o)))
            .map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn list(&self) -> Result<Vec<String>, JsError> {
        self.0.0.list().map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = hasChild)]
    pub fn has_child(&self, name: &str) -> bool {
        self.0.0.has_child(name).unwrap_or(false)
    }

    #[wasm_bindgen(js_name = addChild)]
    pub fn add_child_(&self, name: &str, obj: &FSObj) -> Result<FSObj, JsError> {
        self.0
            .0
            .add_child(name, &obj.0.0)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(FSObj(AssertUnwindSafe(obj.0.0.clone())))
    }

    #[wasm_bindgen(js_name = removeChild)]
    pub fn remove_child_(&self, name: &str) -> Result<(), JsError> {
        self.0
            .0
            .remove_child(name)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = type)]
    pub fn type_(&self) -> String {
        match &**self.0.0.borrow() {
            Object::Int(_) => "Int",
            Object::String(_) => "String",
            Object::Boolean(_) => "Boolean",
            Object::Float(_) => "Float",
            Object::Double(_) => "Double",
            Object::Bytes(_) => "Bytes",
            Object::Null => "Null",
            Object::CompoundFSObj { .. } => "CompoundFSObj",
            Object::Func { .. } => "Func",
        }
        .to_string()
    }

    pub fn string(&self) -> Result<String, JsError> {
        match &**self.0.0.borrow() {
            Object::String(s) => Ok(s.clone()),
            _ => Err(JsError::new("not a string object")),
        }
    }

    pub fn int(&self) -> Result<f64, JsError> {
        match &**self.0.0.borrow() {
            Object::Int(i) => Ok(*i as f64),
            _ => Err(JsError::new("not an int object")),
        }
    }

    pub fn boolean(&self) -> Result<bool, JsError> {
        match &**self.0.0.borrow() {
            Object::Boolean(b) => Ok(*b),
            _ => Err(JsError::new("not a boolean object")),
        }
    }

    pub fn float(&self) -> Result<f64, JsError> {
        match &**self.0.0.borrow() {
            Object::Float(f) => Ok(*f as f64),
            _ => Err(JsError::new("not a float object")),
        }
    }

    pub fn double(&self) -> Result<f64, JsError> {
        match &**self.0.0.borrow() {
            Object::Double(d) => Ok(*d),
            _ => Err(JsError::new("not a double object")),
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>, JsError> {
        match &**self.0.0.borrow() {
            Object::Bytes(b) => Ok(b.clone()),
            _ => Err(JsError::new("not a bytes object")),
        }
    }

    #[wasm_bindgen(js_name = newInt)]
    pub fn new_int(value: f64) -> FSObj {
        FSObj(AssertUnwindSafe(Object::Int(value as i128).into()))
    }

    #[wasm_bindgen(js_name = newString)]
    pub fn new_string(value: String) -> FSObj {
        FSObj(AssertUnwindSafe(Object::String(value).into()))
    }

    #[wasm_bindgen(js_name = newBoolean)]
    pub fn new_boolean(value: bool) -> FSObj {
        FSObj(AssertUnwindSafe(Object::Boolean(value).into()))
    }

    #[wasm_bindgen(js_name = newFloat)]
    pub fn new_float(value: f32) -> FSObj {
        FSObj(AssertUnwindSafe(Object::Float(value).into()))
    }

    #[wasm_bindgen(js_name = newDouble)]
    pub fn new_double(value: f64) -> FSObj {
        FSObj(AssertUnwindSafe(Object::Double(value).into()))
    }

    #[wasm_bindgen(js_name = newBytes)]
    pub fn new_bytes(value: Vec<u8>) -> FSObj {
        FSObj(AssertUnwindSafe(Object::Bytes(value).into()))
    }

    #[wasm_bindgen(js_name = newNull)]
    pub fn new_null() -> FSObj {
        FSObj(AssertUnwindSafe(Object::Null.into()))
    }

    #[wasm_bindgen(js_name = newCompound)]
    pub fn new_compound() -> FSObj {
        FSObj(AssertUnwindSafe(FSObjRef::empty_compound()))
    }

    #[wasm_bindgen(js_name = newFunc)]
    pub fn new_func() -> FSObj {
        FSObj(AssertUnwindSafe(Object::Func(vec![]).into()))
    }

    #[wasm_bindgen(js_name = createInt)]
    pub fn create_int(&self, name: &str, value: f64) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Int(value as i128).into())
    }

    #[wasm_bindgen(js_name = createString)]
    pub fn create_string(&self, name: &str, value: String) -> Result<FSObj, JsError> {
        self.add_child(name, Object::String(value).into())
    }

    #[wasm_bindgen(js_name = createBoolean)]
    pub fn create_boolean(&self, name: &str, value: bool) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Boolean(value).into())
    }

    #[wasm_bindgen(js_name = createFloat)]
    pub fn create_float(&self, name: &str, value: f32) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Float(value).into())
    }

    #[wasm_bindgen(js_name = createDouble)]
    pub fn create_double(&self, name: &str, value: f64) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Double(value).into())
    }

    #[wasm_bindgen(js_name = createBytes)]
    pub fn create_bytes(&self, name: &str, value: Vec<u8>) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Bytes(value).into())
    }

    #[wasm_bindgen(js_name = createNull)]
    pub fn create_null(&self, name: &str) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Null.into())
    }

    #[wasm_bindgen(js_name = createCompound)]
    pub fn create_compound(&self, name: &str) -> Result<FSObj, JsError> {
        let obj = FSObjRef::new_compound(&self.0.0);
        self.add_child(name, obj)
    }

    #[wasm_bindgen(js_name = createFunc)]
    pub fn create_func(&self, name: &str) -> Result<FSObj, JsError> {
        self.add_child(name, Object::Func(vec![]).into())
    }

    #[wasm_bindgen(js_name = ensureInt)]
    pub fn ensure_int(&self, name: &str, value: f64) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_int(name, value),
        }
    }

    #[wasm_bindgen(js_name = ensureString)]
    pub fn ensure_string(&self, name: &str, value: String) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_string(name, value),
        }
    }

    #[wasm_bindgen(js_name = ensureBoolean)]
    pub fn ensure_boolean(&self, name: &str, value: bool) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_boolean(name, value),
        }
    }

    #[wasm_bindgen(js_name = ensureFloat)]
    pub fn ensure_float(&self, name: &str, value: f32) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_float(name, value),
        }
    }

    #[wasm_bindgen(js_name = ensureDouble)]
    pub fn ensure_double(&self, name: &str, value: f64) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_double(name, value),
        }
    }

    #[wasm_bindgen(js_name = ensureBytes)]
    pub fn ensure_bytes(&self, name: &str, value: Vec<u8>) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_bytes(name, value),
        }
    }

    #[wasm_bindgen(js_name = ensureNull)]
    pub fn ensure_null(&self, name: &str) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_null(name),
        }
    }

    #[wasm_bindgen(js_name = ensureCompound)]
    pub fn ensure_compound(&self, name: &str) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_compound(name),
        }
    }

    #[wasm_bindgen(js_name = ensureFunc)]
    pub fn ensure_func(&self, name: &str) -> Result<FSObj, JsError> {
        match self.0.0.get_obj(name) {
            Ok(obj) => Ok(FSObj(AssertUnwindSafe(obj))),
            Err(_) => self.create_func(name),
        }
    }
}

#[wasm_bindgen]
pub struct LuaEnv(AssertUnwindSafe<mlua::Lua>);

#[wasm_bindgen]
impl LuaEnv {
    #[wasm_bindgen(constructor)]
    pub fn new() -> LuaEnv {
        LuaEnv(AssertUnwindSafe(mlua::Lua::new()))
    }

    pub fn run(&self, code: &str) -> Result<(), JsError> {
        self.0
            .0
            .load(code)
            .exec()
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = createProcess)]
    pub fn create_process(&self, name: &str, code: &str) -> Result<u64, JsError> {
        let func = self
            .0
            .0
            .load(code)
            .set_name(name)
            .into_function()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let process =
            cworks_lua::new_lua_process(func).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(with_session(|k| k.register_process(Box::new(process))))
    }
}

struct JsProcess {
    callback: js_sys::Function,
    pending: PendingRequest,
}

impl Process for JsProcess {
    fn poll(&mut self, data: &SyscallData) -> PollResult {
        self.pending.clear();
        let payload = syscall_data_to_js(data);
        let _ = self.callback.call1(&JsValue::NULL, &payload);
        self.pending.take()
    }
}

fn syscall_data_to_js(data: &SyscallData) -> JsValue {
    let obj = js_sys::Object::new();
    let set = |key: &str, value: &JsValue| {
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(key), value);
    };
    match data {
        SyscallData::None => set("kind", &JsValue::from_str("none")),
        SyscallData::Fail(e) => {
            set("kind", &JsValue::from_str("fail"));
            set("error", &JsValue::from_str(&e.to_string()));
        }
        SyscallData::FSRoot(root) => {
            set("kind", &JsValue::from_str("fsRoot"));
            set("obj", &JsValue::from(FSObj(AssertUnwindSafe(root.clone()))));
        }
        SyscallData::FSSuccess => set("kind", &JsValue::from_str("success")),
        SyscallData::Invoke {
            caller_pid,
            obj: channel,
            arg,
        } => {
            set("kind", &JsValue::from_str("invoke"));
            set("callerPid", &JsValue::from_f64(*caller_pid as f64));
            set(
                "obj",
                &JsValue::from(FSObj(AssertUnwindSafe(channel.clone()))),
            );
            set("arg", &JsValue::from(FSObj(AssertUnwindSafe(arg.clone()))));
        }
        SyscallData::GetPid(pid) => {
            set("kind", &JsValue::from_str("getPid"));
            set("pid", &JsValue::from_f64(*pid as f64));
        }
    }
    obj.into()
}
