use wasm_bindgen::prelude::wasm_bindgen;

use crate::js_process::CallbackProcess;
#[wasm_bindgen]
pub struct Session {
    #[wasm_bindgen(skip)]
    pub kernel: kernel::Kernel,
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

    pub fn add_process(&mut self, callback: js_sys::Function) -> u128 {
        let p = Box::new(CallbackProcess::new(callback));
        self.kernel.register_process(p)
    }

    pub fn step(&mut self) {
        self.kernel.step();
    }
}
