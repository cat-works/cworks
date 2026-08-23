use kernel::Kernel;
use std::cell::RefCell;

use crate::js_callback::call_js_callback;

struct JsCallbackProcess {
    callback_id: i32,
}

impl kernel::Process for JsCallbackProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult {
        let json = serde_json::to_string(data).unwrap_or_default();
        // Wrap with callback ID so JS dispatcher can route correctly
        let wrapped = format!(
            "{{\"id\":{},\"data\":{}}}",
            self.callback_id, json
        );
        match call_js_callback(self.callback_id, &wrapped) {
            Some(result_json) => {
                serde_json::from_str(&result_json).unwrap_or(kernel::PollResult::Done)
            }
            None => kernel::PollResult::Done,
        }
    }
}

thread_local! {
    static SESSION: RefCell<Option<Kernel>> = RefCell::new(None);
}

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_session_new() {
    SESSION.with(|s| {
        *s.borrow_mut() = Some(Kernel::default());
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_session_add_process(callback_id: i32) -> u64 {
    SESSION.with(|s| {
        let mut sess = s.borrow_mut();
        let kernel = sess.as_mut().unwrap();
        let process = Box::new(JsCallbackProcess { callback_id });
        kernel.register_process(process) as u64
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_session_step() {
    SESSION.with(|s| {
        if let Some(ref mut kernel) = *s.borrow_mut() {
            kernel.step();
        }
    });
}
