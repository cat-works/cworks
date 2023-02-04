use kernel::{PollResult, Process, Syscall, SyscallData};

extern crate kernel;
extern crate python;

mod processes;

const IPC_NAME: &str = "0syoch/test-ipc";

struct IPC_master {
    state: i32,
    ipc_handle: u128,
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
                    self.ipc_handle = handle.id;
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
        ipc_handle: 0,
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
