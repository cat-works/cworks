use std::sync::Arc;

use kernel::{Handle, PollResult, Process, Syscall, SyscallData};
#[derive(Default)]
pub struct IPCSlave {
    state: i32,
    dest: Arc<Handle>,
}

impl Process for IPCSlave {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        match self.state {
            0 => {
                self.state = 1;
                PollResult::Sleep(0.1)
            }

            1 => {
                self.state = 2;
                println!("[Slave ] Connecting..");
                PollResult::Syscall(Syscall::IpcConnect("system/test_ipc".to_string()))
            }
            2 => {
                self.state = 3;
                if let SyscallData::Handle(Ok(handle)) = data {
                    self.dest = handle.clone();
                    println!("[Slave ] IPC Connected: {}", self.dest);
                    PollResult::Pending
                } else if let SyscallData::Handle(Err(e)) = data {
                    println!("[Slave ] IPC Connect error: {:?}", e);
                    PollResult::Done(-1)
                } else {
                    println!("[Slave ] Invalid state: {:?}", data);
                    panic!("Invalid state");
                }
            }

            3 => {
                self.state = 4;
                PollResult::Syscall(Syscall::Send(
                    self.dest.clone(),
                    "Hello, world!".to_string(),
                ))
            }

            4 => {
                if let SyscallData::ReceivingData {
                    focus: client,
                    data,
                } = data
                {
                    if client != &self.dest {
                        println!("Invalid client handle");
                        println!("  Required: {}", self.dest);
                        println!("  Actual: {}", client);
                        panic!("Invalid client handle");
                    }
                    println!("[Slave ] Received: {}", data);
                    PollResult::Done(0)
                } else {
                    PollResult::Pending
                }
            }

            _ => {
                panic!("Invalid state");
            }
        }
    }
}
