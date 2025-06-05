use js_sys::Uint8Array;
use kernel::{PollResult, Process, Syscall, SyscallData, SyscallError};
use log::{debug, error};
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

    fn syscall_data_to_vec_u8(&mut self, value: SyscallData) -> Vec<u8> {
        match value {
            SyscallData::None => vec![0x00],
            SyscallData::Fail(SyscallError::AlreadyExists) => vec![0x01],
            SyscallData::Fail(SyscallError::NoSuchEntry) => vec![0x02],
            SyscallData::Fail(SyscallError::NotImplemented) => vec![0x03],
            SyscallData::Fail(SyscallError::ResourceIsBusy) => vec![0x04],
            SyscallData::Fail(SyscallError::UnknownHandle) => vec![0x05],
            SyscallData::Fail(SyscallError::UnreachableEntry) => vec![0x06],
            SyscallData::Handle(handle) => {
                self.handle_casher.register_handle(handle.clone());

                let mut a = vec![0x07];
                a.extend(handle.id.to_be_bytes());
                a
            }
            SyscallData::Connection { client, server } => {
                self.handle_casher.register_handle(client.clone());
                self.handle_casher.register_handle(server.clone());

                let mut a = vec![0x08];
                a.extend(client.id.to_be_bytes());
                a.extend(server.id.to_be_bytes());

                a
            }
            SyscallData::ReceivingData { focus, data } => {
                self.handle_casher.register_handle(focus.clone());

                let mut a = vec![0x09];
                a.extend(focus.id.to_be_bytes());
                a.extend(data.into_bytes());
                a
            }
        }
    }

    fn vec_u8_to_poll_result(&mut self, value: Vec<u8>) -> PollResult<i64> {
        match value[0] {
            0x00 => PollResult::Pending,
            0x01 => {
                let return_value = i64::from_be_bytes(
                    value[1..9]
                        .try_into()
                        .expect("Failed to convert bytes to i64"),
                );
                PollResult::Done(return_value)
            }
            0x02 => {
                // 1..5: f32 (duration)
                let duration = f32::from_be_bytes(
                    value[1..5]
                        .try_into()
                        .expect("Failed to convert bytes to f32"),
                );
                PollResult::Syscall(kernel::Syscall::Sleep(duration))
            }
            0x03 => {
                let ipc_name = String::from_utf8(value[1..].to_vec())
                    .expect("Failed to convert bytes to String");
                PollResult::Syscall(kernel::Syscall::IpcCreate(ipc_name))
            }
            0x04 => {
                let ipc_name = String::from_utf8(value[1..].to_vec())
                    .expect("Failed to convert bytes to String");
                PollResult::Syscall(kernel::Syscall::IpcConnect(ipc_name))
            }
            0x05 => {
                // 1..17: u128 (handle ID)
                // 17..: String (data)

                let handle_id = u128::from_be_bytes(
                    value[1..17]
                        .try_into()
                        .expect("Failed to convert bytes to u128"),
                );
                let handle = self.handle_casher.get_handle(handle_id).unwrap();

                let data = String::from_utf8(value[17..].to_vec())
                    .expect("Failed to convert bytes to String");

                PollResult::Syscall(kernel::Syscall::Send(handle, data))
            }
            _ => panic!("Unknown syscall data type"),
        }
    }
}

impl Process for CallbackProcess {
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult<i64> {
        let syscall_data = if let Some(d) = self.syscall_data_buffer.take() {
            d
        } else {
            data.clone()
        };
        let syscall_data = self.syscall_data_to_vec_u8(syscall_data);
        // if syscall_data[0] != 0x00 {
        //     debug!("Syscall data: {:?}", syscall_data);
        // }
        let data = Uint8Array::from(syscall_data.as_slice());

        let this = JsValue::null();
        let ret: Result<JsValue, JsValue> = self.callback.call1(&this, &data);
        ret.map_or_else(
            |err| {
                error!("Callback error: {:?}", err);
                PollResult::Done(-1)
            },
            |result| {
                let arr = Uint8Array::new(&result);
                let result = arr.to_vec();
                // if result[0] != 0x00 {
                //     debug!("Poll: {:?}", result);
                // }
                self.vec_u8_to_poll_result(result)
            },
        )
    }
}
