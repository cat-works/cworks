mod generator;
mod processes;

extern crate ghs_demangle;

use std::collections::HashMap;

pub use generator::generate_user_id;
use kernel::{Handle, PollResult, Process, Syscall, SyscallData};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);

    #[wasm_bindgen(js_namespace = console, js_name = clog)]
    fn log2(a: JsValue);

    #[wasm_bindgen(js_name = my_stringify)]
    fn stringify(a: JsValue) -> JsValue;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Default)]
struct HandleCasher {
    handles: HashMap<u128, Handle>,
}

impl HandleCasher {
    pub fn register_handle(&mut self, h: Handle) {
        let h = h.clone();
        // console_log!("Registering Handle {}", h.clone());
        self.handles.insert(h.id, h);
    }

    pub fn get_handle(&self, id: u128) -> Option<Handle> {
        self.handles.get(&id).cloned()
    }
}

#[derive(Default)]
struct CallbackProcess {
    callback: js_sys::Function,
    handle_casher: HandleCasher,
    syscall_data_buffer: Option<SyscallData>,
}

impl CallbackProcess {
    pub fn new(callback: js_sys::Function) -> Self {
        Self {
            callback,
            ..Default::default()
        }
    }
}

impl Process for CallbackProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult<i64> {
        if let SyscallData::Handle(ref h) = data {
            self.handle_casher.register_handle(h.clone());
        }
        if let SyscallData::Connection {
            ref server,
            ref client,
        } = data
        {
            self.handle_casher.register_handle(server.clone());
            self.handle_casher.register_handle(client.clone());
        }

        let data = {
            match serde_wasm_bindgen::to_value(&{
                if let Some(d) = self.syscall_data_buffer.take() {
                    d.clone()
                } else {
                    data.clone()
                }
            }) {
                Ok(v) => v,
                Err(_) => {
                    return kernel::PollResult::Done(-1);
                }
            }
        };
        // console_log!("Passing {:?}", stringify(data.clone()));

        let this = JsValue::null();
        let ret = self
            .callback
            .call1(&this, &data)
            .map(serde_wasm_bindgen::from_value::<PollResult<i64>>);

        let ret = match ret {
            Ok(Ok(x)) => match x {
                PollResult::Syscall(Syscall::Send(h, d)) => {
                    if let Some(h) = self.handle_casher.get_handle(h.id) {
                        PollResult::Syscall(Syscall::Send(h.clone(), d))
                    } else {
                        self.syscall_data_buffer =
                            Some(SyscallData::Fail(kernel::SyscallError::UnknownHandle));
                        PollResult::Pending
                    }
                }
                _ => x,
            },
            _ => PollResult::Done(-1),
        };

        // let r = serde_wasm_bindgen::to_value(&ret.clone());
        // if let Ok(r) = r {
        //      console_log!("Got {:?}", stringify(r));
        // }

        return ret;
    }
}

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
