use std::sync::Arc;

use kernel::{Handle, PollResult, Process, Syscall, SyscallData};

#[derive(Default)]
pub struct IPCMaster {
    state: i32,
    ipc_handle: Arc<Handle>,
    client_handle: Arc<Handle>,
}

impl Process for IPCMaster {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        match self.state {
            0 => {
                self.state = 1;
                println!("[Master] Creating...");
                PollResult::Syscall(Syscall::IpcCreate("system/test_ipc".to_string()))
            }

            1 => {
                if let SyscallData::Handle(Ok(handle)) = data {
                    self.ipc_handle = handle.clone();
                    self.state = 2;
                    println!("[Master] IPC created: {}", self.ipc_handle);
                    PollResult::Pending
                } else if let SyscallData::Handle(Err(e)) = data {
                    println!("[Master] IPC create error: {:?}", e);
                    PollResult::Done(-1)
                } else {
                    panic!("Invalid state");
                }
            }

            2 => {
                let s = &self.ipc_handle;
                if let SyscallData::Connection { client, server } = data {
                    if server != s {
                        panic!("Invalid server handle");
                    }
                    self.client_handle = client.clone();
                    println!("[Master] IPC connection coming: {}", client);
                    self.state = 3;
                    PollResult::Pending
                } else {
                    PollResult::Pending
                }
            }

            3 => {
                if let SyscallData::ReceivingData {
                    focus: client,
                    data,
                } = data
                {
                    if client != &self.client_handle {
                        panic!("Invalid client handle");
                    }
                    println!("[Master] Received: {}", data);
                    self.state = 4;
                    PollResult::Syscall(Syscall::Send(
                        self.client_handle.clone(),
                        "Hello, world!".to_string(),
                    ))
                } else {
                    PollResult::Pending
                }
            }
            4 => PollResult::Done(0),

            _ => {
                println!("[Master] Invalid state: {:?} {}", data, self.state);
                panic!("Invalid state");
            }
        }
    }
}
