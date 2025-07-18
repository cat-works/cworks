use std::cell::RefCell;

use kernel::Process;
use wasm_bindgen::prelude::wasm_bindgen;

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

    pub fn add_process(&self, callback: js_sys::Function) {
        self.spawn_queue
            .borrow_mut()
            .push(Box::new(CallbackProcess::new(callback)));
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
