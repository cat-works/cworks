use std::{cell::RefCell, rc::Rc};

use kernel::Process;
use lua::{LuaEnv, LuaProcess};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::js_process::CallbackProcess;
#[wasm_bindgen]
pub struct Session {
    #[wasm_bindgen(skip)]
    pub kernel: RefCell<kernel::Kernel>,
    #[wasm_bindgen(skip)]
    pub lua: Rc<LuaEnv>,

    spawn_queue: RefCell<Vec<Box<dyn Process>>>,
}

#[wasm_bindgen]
impl Session {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            kernel: RefCell::new(kernel::Kernel::default()),
            lua: Rc::new(LuaEnv::new()),
            spawn_queue: RefCell::new(Vec::new()),
        }
    }

    pub fn add_process(&self, callback: js_sys::Function) {
        self.spawn_queue
            .borrow_mut()
            .push(Box::new(CallbackProcess::new(callback)));
    }

    pub fn add_lua_process(&self, code: js_sys::JsString) {
        self.spawn_queue
            .borrow_mut()
            .push(Box::new(LuaProcess::new(self.lua.clone(), code.into())));
    }

    pub fn step(&self) {
        for p in self.spawn_queue.borrow_mut().drain(..) {
            self.kernel.borrow_mut().register_process(p);
        }
        self.kernel.borrow_mut().step();
    }

    pub fn get_ipc_names(&self) -> Vec<String> {
        self.kernel.borrow().get_ipc_names()
    }
}
