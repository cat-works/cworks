use std::cell::RefCell;

use kernel::Process;
use python::PythonProcess;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::js_process::CallbackProcess;
#[wasm_bindgen]
pub struct Session {
    #[wasm_bindgen(skip)]
    pub kernel: RefCell<kernel::Kernel>,
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

    pub fn add_python_process(&self, code: String) -> Option<String> {
        let p = PythonProcess::new(code);
        match p {
            Err(_) => Some("Fail".to_string()),
            Ok(r) => {
                self.spawn_queue.borrow_mut().push(Box::new(r));
                None
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
