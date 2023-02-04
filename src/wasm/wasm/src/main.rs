use kernel::{Handle, PollResult, Process, Syscall, SyscallData};

extern crate kernel;
extern crate python;

mod processes;

const IPC_NAME: &str = "0syoch/test-ipc";

struct IPC_master {
    state: i32,
    ipc_handle: Handle,
    client_handle: Handle,
}

impl Process for IPC_master {
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
                    println!("IPC created: {}", self.ipc_handle);
                    PollResult::Pending
                } else if let SyscallData::Handle(Err(e)) = data {
                    println!("IPC create error: {:?}", e);
                    PollResult::Done(-1)
                } else {
                    panic!("Invalid state");
                }
            }

            2 => {
                let s = &self.ipc_handle;
                if let SyscallData::Connection {
                    client: client,
                    server: server,
                } = data
                {
                    if server != s {
                        panic!("Invalid server handle");
                    }
                    println!("IPC connection coming: {}", client);
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

struct IPC_slave {
    state: i32,
    ipc_handle: Handle,
}

impl Process for IPC_slave {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        match self.state {
            0 => {
                self.state = 1;
                println!("s: sleep 1.0secs");
                PollResult::Sleep(1.0)
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
                if let SyscallData::ReceivingData {
                    client: client,
                    data: data,
                } = data
                {
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
    let p = IPC_master {
        state: 0,
        ipc_handle: Handle::new(0),
        client_handle: Handle::new(0),
    };
    k.register_process(Box::new(p));

    let p = IPC_slave {
        state: 0,
        ipc_handle: Handle::new(0),
    };
    k.register_process(Box::new(p));
    k.start();
    Ok(())
}
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
