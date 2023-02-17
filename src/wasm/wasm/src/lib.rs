mod generator;
mod processes;

extern crate ghs_demangle;

use std::collections::HashMap;

pub use generator::generate_user_id;
use kernel::{
    Handle, HandleData, HandleIssuer, PollResult, Process, Syscall, SyscallData, SyscallError,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);

    #[wasm_bindgen(js_namespace = console, js_name = clog)]
    fn log2(a: JsValue);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

struct CallbackProcess {
    callback: js_sys::Function,
    handles: HashMap<u128, Handle>,
}

impl CallbackProcess {
    pub fn new(callback: js_sys::Function) -> Self {
        Self {
            callback,
            handles: HashMap::default(),
        }
    }
}

impl Process for CallbackProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult<i64> {
        if let SyscallData::Handle(ref h) = data {
            let h = h.clone();
            self.handles.insert(h.id, h);
        }

        let data = {
            let r = serde_wasm_bindgen::to_value(data);
            match r {
                Ok(v) => v,
                Err(_) => {
                    return kernel::PollResult::Done(-1);
                }
            }
        };

        let this = JsValue::null();
        let ret = self
            .callback
            .call1(&this, &data)
            .map(serde_wasm_bindgen::from_value::<PollResult<i64>>);
        match ret {
            Ok(Ok(x)) => match x {
                PollResult::Syscall(Syscall::Send(h, d)) => {
                    PollResult::Syscall(Syscall::Send(self.handles.get(&h.id).unwrap().clone(), d))
                }
                _ => x,
            },
            _ => PollResult::Done(-1),
        }
    }
}

#[wasm_bindgen]
pub struct Session {
    kernel: kernel::Kernel,
}

#[wasm_bindgen]
impl Session {
    #[allow(clippy::new_without_default)]
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
        self.kernel
            .register_process(Box::new(CallbackProcess::new(callback)));
        JsValue::from_str("aiueo")
    }

    pub fn step(&mut self) {
        self.kernel.step();
    }
}

#[wasm_bindgen]
pub fn demangle_str(x: String) -> String {
    ghs_demangle::demangle(x).to_string()
}

#[wasm_bindgen(start)]
fn m() {
    // let s = serde_wasm_bindgen::to_value(&PollResult::<i64>::Syscall(Syscall::Send(
    //     HandleIssuer::default().get_new_handle(0, HandleData::None),
    //     "aiu".to_string(),
    // )));
    // log2(s.unwrap());
    // console_log!("{:?}", s.unwrap());
}
