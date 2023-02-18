use kernel::Process;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::js_process::CallbackProcess;

#[wasm_bindgen]
pub struct Session {
    kernel: kernel::Kernel,
    spawn_queue: Vec<Box<dyn Process>>,
}

#[wasm_bindgen]
impl Session {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            kernel: kernel::Kernel::default(),
            spawn_queue: Vec::default(),
        }
    }

    /* pub fn add_python_process(&mut self, code: String) -> JsValue {
        let p = PythonProcess::new(code);
        match p {
            Err(e) => python_enter(|vm| {
                JsValue::from_str(&format!(
                    "Failed to Create Process: {}",
                    python::format_exception(e, vm).to_string()
                ))
            }),
            Ok(r) => {
                self.kernel.register_process(Box::new(r));
                JsValue::from_str("Success")
            }
        }
    } */

    pub fn add_process(&mut self, callback: js_sys::Function) -> JsValue {
        self.spawn_queue
            .push(Box::new(CallbackProcess::new(callback)));

        JsValue::from_str("aiueo")
    }

    pub fn step(&mut self) {
        for p in self.spawn_queue.drain(..) {
            self.kernel.register_process(p);
        }
        self.kernel.step();
    }
}
