use std::cell::RefCell;

use kernel::Process;
use python::{python_enter, PythonProcess};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::js_process::CallbackProcess;
#[wasm_bindgen]
pub struct Session {
    kernel: RefCell<kernel::Kernel>,
    spawn_queue: RefCell<Vec<Box<dyn Process>>>,
}

#[wasm_bindgen]
impl Session {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            kernel: RefCell::new(kernel::Kernel::default()),
            spawn_queue: RefCell::new(Vec::new()),
        }
    }

    pub fn add_python_process(&self, code: String) -> JsValue {
        let p = PythonProcess::new(code);
        match p {
            Err(e) => {
                error!(format!("Failed to create Python process: {e}"));
                JsValue::UNDEFINED
            }
            Ok(r) => {
                self.spawn_queue.borrow_mut().push(Box::new(r));
                JsValue::from_str("Python process added")
            }
        }
    }

    pub fn add_process(&self, callback: js_sys::Function) -> JsValue {
        self.spawn_queue
            .borrow_mut()
            .push(Box::new(CallbackProcess::new(callback)));

        JsValue::from_str("aiueo")
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
