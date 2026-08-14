use kernel::{PollResult, Process};
use log::error;
use wasm_bindgen::JsValue;

#[derive(Default)]
pub struct CallbackProcess {
    callback: js_sys::Function,
}

impl CallbackProcess {
    pub const fn new(callback: js_sys::Function) -> Self {
        Self { callback }
    }
}

impl Process for CallbackProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult<i64> {
        let data = serde_wasm_bindgen::to_value(data);
        if data.is_err() {
            error!("Failed to serialize SyscallData: {:?}", data.err());
            return PollResult::Done(-1);
        }
        let data = data.unwrap();

        let this = JsValue::null();
        let ret: Result<JsValue, JsValue> = self.callback.call1(&this, &data);
        if ret.is_err() {
            error!("Failed to call callback: {:?}", ret.err());
            return PollResult::Done(-1);
        }
        let ret = ret.unwrap();

        let result = serde_wasm_bindgen::from_value::<PollResult<i64>>(ret);
        if result.is_err() {
            error!("Failed to deserialize PollResult: {:?}", result.err());
            return PollResult::Done(-1);
        }

        result.unwrap()
    }
}
