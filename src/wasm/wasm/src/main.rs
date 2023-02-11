use std::sync::Arc;

use kernel::{Handle, PollResult, Process, Syscall, SyscallData};

extern crate kernel;
// extern crate python;

// mod processes;

const IPC_NAME: &str = "0syoch/test-ipc";

struct IPCMaster {
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
                PollResult::Syscall(Syscall::IpcCreate(IPC_NAME.to_string()))
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

struct IPCSlave {
    state: i32,
    dest: Arc<Handle>,
}

impl Process for IPCSlave {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        match self.state {
            0 => {
                self.state = 1;
                PollResult::Sleep(0.5)
            }

            1 => {
                self.state = 2;
                println!("[Slave ] Connecting..");
                PollResult::Syscall(Syscall::IpcConnect(IPC_NAME.to_string()))
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();
    let p = IPCMaster {
        state: 0,
        ipc_handle: Handle::default().into(),
        client_handle: Handle::default().into(),
    };
    k.register_process(Box::new(p));

    let p = IPCSlave {
        state: 0,
        dest: Handle::default().into(),
    };
    k.register_process(Box::new(p));
    k.start();
    Ok(())
}
/*
#[cfg(test)]
mod test {
    use python::PythonProcess;

    #[test]
    fn python_based_process() {
        let mut k = kernel::Kernel::default();
        let p = PythonProcess::new(
            r#"async def proc():
        print("print")
        print("/")

        print("step")
        await step()

        print("pending")
        await pending()"#
                .to_string(),
        )
        .unwrap();
        k.register_process(Box::new(p));
        k.start();
    }
}

*/
