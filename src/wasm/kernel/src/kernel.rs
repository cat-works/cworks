use std::collections::HashMap;

use crate::{
    fs::RefOrVal,
    handle::{HandleData, HandleIssuer},
    ipc::Ipc,
    libs::{timestamp, AutoMap},
    process::{ProcessStatus, Syscall, SyscallData, SyscallError},
};

use super::{
    fs::FSObj,
    process::{KernelProcess, PollResult, Process},
};

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    fs_root: FSObj,
    ipc_instances: HashMap<String, Ipc>,
    handle_issuer: HandleIssuer,
}

impl Default for Kernel {
    fn default() -> Kernel {
        Kernel {
            processes: AutoMap::new(),
            fs_root: FSObj::Dist(RefOrVal::Val(HashMap::new())),
            ipc_instances: HashMap::new(),
            handle_issuer: HandleIssuer::default(),
        }
    }
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) {
        self.processes.add_value(p.into());
    }

    pub fn start(&mut self) {
        while !self.processes.map.is_empty() {
            let mut process_to_kill = vec![];
            for (pid, p) in &mut self.processes.map {
                if let ProcessStatus::Sleeping(t) = p.status {
                    if t > timestamp() {
                        continue;
                    } else {
                        p.status = ProcessStatus::Running;
                    }
                }

                let data = p.outgoing_data_buffer.pop().unwrap_or(SyscallData::None);

                match p.process.poll(&data) {
                    PollResult::Pending => (),
                    PollResult::Done(n) => {
                        println!("Process<{pid}> Returns {n}");
                        process_to_kill.push(*pid);
                    }
                    PollResult::Sleep(n) => {
                        p.status = ProcessStatus::Sleeping(timestamp() + n);
                    }
                    PollResult::Syscall(s) => {
                        match s {
                            Syscall::IPC_Create(ref name) => {
                                if self.ipc_instances.contains_key(name) {
                                    p.outgoing_data_buffer.push(SyscallData::Handle(Err(
                                        SyscallError::AlreadyExists,
                                    )));
                                    break;
                                }

                                let handle = self
                                    .handle_issuer
                                    .get_new_handle(HandleData::IpcServer(name.to_string()));

                                let ipc = Ipc::new(handle.clone());
                                self.ipc_instances.insert(name.clone(), ipc);

                                p.outgoing_data_buffer.push(SyscallData::Handle(Ok(handle)));
                                break;
                            }
                            Syscall::IPC_Connect(ref name) => {
                                if !self.ipc_instances.contains_key(name) {
                                    p.outgoing_data_buffer
                                        .push(SyscallData::Handle(Err(SyscallError::NoSuchEntry)));
                                    break;
                                }

                                let handle = self
                                    .handle_issuer
                                    .get_new_handle(HandleData::IpcClient(name.to_string()));

                                let ipc = self.ipc_instances.get_mut(name).unwrap();
                                ipc.connect(handle.clone());

                                let server = ipc.get_server_handle();

                                p.outgoing_data_buffer.push(SyscallData::Handle(Ok(handle)));
                                break;
                            }
                            Syscall::Send(ref handle, ref data) => {
                                // TODO: IPC Send
                                p.outgoing_data_buffer
                                    .push(SyscallData::Handle(Err(SyscallError::NotImplemented)));
                            }
                        }
                        println!("{pid}: {s:?}");
                    }
                }
            }

            for pid in process_to_kill {
                self.processes.map.remove(&pid);
            }
        }
    }
}
