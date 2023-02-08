use kernel::{Handle, PollResult, Process, Syscall, SyscallData};

extern crate kernel;
// extern crate python;

// mod processes;

const IPC_NAME: &str = "0syoch/test-ipc";

struct IPCMaster {
    state: i32,
    ipc_handle: Handle,
    client_handle: Handle,
}

impl Process for IPCMaster {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        match self.state {
            0 => {
                self.state = 1;
                PollResult::Syscall(Syscall::IPC_Create(IPC_NAME.to_string()))
            }

            1 => {
                if let SyscallData::Handle(Ok(handle)) = data {
                    self.ipc_handle = handle.clone();
                    self.state = 2;
                    println!("m: IPC created: {}", self.ipc_handle);
                    PollResult::Pending
                } else if let SyscallData::Handle(Err(e)) = data {
                    println!("m: IPC create error: {:?}", e);
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
                    println!("m: IPC connection coming: {}", client);
                    self.state = 3;
                    PollResult::Pending
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

struct IPCSlave {
    state: i32,
    ipc_handle: Handle,
}

impl Process for IPCSlave {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        match self.state {
            0 => {
                self.state = 1;
                println!("s: sleep 1.0secs");
                PollResult::Sleep(0.5)
            }

            1 => {
                self.state = 2;
                println!("s: wake?");
                PollResult::Syscall(Syscall::IPC_Connect(IPC_NAME.to_string()))
            }
            2 => {
                if let SyscallData::Handle(Ok(handle)) = data {
                    self.ipc_handle = handle.clone();
                    self.state = 3;
                    println!("s: IPC Connected: {}", self.ipc_handle);
                    PollResult::Pending
                } else if let SyscallData::Handle(Err(e)) = data {
                    println!("s: IPC Connect error: {:?}", e);
                    PollResult::Done(-1)
                } else {
                    println!("s: Invalid state: {:?}", data);
                    panic!("Invalid state");
                }
            }

            3 => {
                self.state = 4;
                PollResult::Syscall(Syscall::Send(
                    self.ipc_handle.clone(),
                    "Hello, world!".to_string(),
                ))
            }

            4 => {
                if let SyscallData::ReceivingData { client, data } = data {
                    if client != &self.ipc_handle {
                        panic!("Invalid client handle");
                    }
                    println!("s: Received: {}", data);
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
        ipc_handle: Handle::default(),
        client_handle: Handle::default(),
    };
    k.register_process(Box::new(p));

    let p = IPCSlave {
        state: 0,
        ipc_handle: Handle::default(),
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
