mod generator;
mod processes;

extern crate ghs_demangle;

use std::sync::{Arc, Mutex};

pub use generator::generate_user_id;
use kernel::Process;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

struct CallbackProcess {
    callback: Arc<Mutex<js_sys::Function>>
}

impl CallbackProcess {
    pub fn new(callback: Arc<Mutex<js_sys::Function>>) -> Self {
        Self {
            callback
        }
    }
}

impl Process for CallbackProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult<i64> {
        data
    }
}

#[wasm_bindgen]
pub struct Session {
    kernel: kernel::Kernel,
}

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            kernel: kernel::Kernel::default(),
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
        callback.call

        JsValue::from_str("aiueo")
    }
}

#[wasm_bindgen]
pub fn demangle_str(x: String) -> String {
    ghs_demangle::demangle(x).to_string()
}
