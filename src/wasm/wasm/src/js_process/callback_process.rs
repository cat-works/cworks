use kernel::{PollResult, Process, Syscall, SyscallData};
use wasm_bindgen::JsValue;

use super::handle_casher::HandleCasher;

#[derive(Default)]
pub struct CallbackProcess {
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
                    d
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

        match ret {
            Ok(Ok(x)) => match x {
                PollResult::Syscall(Syscall::Send(h, d)) => {
                    if let Some(h) = self.handle_casher.get_handle(h.id) {
                        PollResult::Syscall(Syscall::Send(h, d))
                    } else {
                        self.syscall_data_buffer =
                            Some(SyscallData::Fail(kernel::SyscallError::UnknownHandle));
                        PollResult::Pending
                    }
                }
                _ => x,
            },
            _ => PollResult::Done(-1),
        }
    }
}
