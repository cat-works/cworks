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

enum KernelAction {
    ProcessKill(u128),
    SendSyscallData(u128, SyscallData),
}

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    fs_root: FSObj,
    ipc_instances: HashMap<String, Ipc>,
    handle_issuer: HandleIssuer,
    actions: Vec<KernelAction>,
}

impl Default for Kernel {
    fn default() -> Kernel {
        Kernel {
            processes: AutoMap::new(),
            fs_root: FSObj::Dist(RefOrVal::Val(HashMap::new())),
            ipc_instances: HashMap::new(),
            handle_issuer: HandleIssuer::default(),
            actions: vec![],
        }
    }
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) {
        self.processes.add_value(p.into());
    }

    fn syscall(&mut self, pid: u128) {}

    fn step_all_processes(&mut self) {
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
                    self.actions.push(KernelAction::ProcessKill(*pid));
                }
                PollResult::Sleep(n) => {
                    p.status = ProcessStatus::Sleeping(timestamp() + n);
                }
                PollResult::Syscall(s) => {
                    match s {
                        Syscall::IPC_Create(ref name) => {
                            if self.ipc_instances.contains_key(name) {
                                p.outgoing_data_buffer
                                    .push(SyscallData::Handle(Err(SyscallError::AlreadyExists)));
                                continue;
                            }

                            let handle = self
                                .handle_issuer
                                .get_new_handle(*pid, HandleData::IpcServer(name.to_string()));

                            let ipc = Ipc::new(handle.clone());
                            self.ipc_instances.insert(name.clone(), ipc);

                            p.outgoing_data_buffer.push(SyscallData::Handle(Ok(handle)));
                            continue;
                        }
                        Syscall::IPC_Connect(ref name) => {
                            if !self.ipc_instances.contains_key(name) {
                                p.outgoing_data_buffer
                                    .push(SyscallData::Handle(Err(SyscallError::NoSuchEntry)));
                                continue;
                            }

                            let handle = self
                                .handle_issuer
                                .get_new_handle(*pid, HandleData::IpcClient(name.to_string()));

                            let ipc = self.ipc_instances.get_mut(name).unwrap();
                            ipc.connect(handle.clone());

                            let server = ipc.get_server_handle();
                            self.actions.push(KernelAction::SendSyscallData(
                                server.pid,
                                SyscallData::Connection {
                                    client: handle.clone(),
                                    server: server.clone(),
                                },
                            ));

                            p.outgoing_data_buffer.push(SyscallData::Handle(Ok(handle)));
                            continue;
                        }
                        Syscall::Send(ref client, ref handle, ref data) => {
                            // TODO: IPC Send
                            p.outgoing_data_buffer
                                .push(SyscallData::Handle(Err(SyscallError::NotImplemented)));
                        }
                    }
                    println!("{pid}: {s:?}");
                }
            }
        }
    }

    pub fn start(&mut self) {
        while !self.processes.map.is_empty() {
            self.actions = vec![];

            self.step_all_processes();

            while let Some(act) = self.actions.pop() {
                match act {
                    KernelAction::ProcessKill(pid) => {
                        self.processes.map.remove(&pid);
                    }
                    KernelAction::SendSyscallData(pid, data) => {
                        self.processes
                            .map
                            .get_mut(&pid)
                            .unwrap()
                            .outgoing_data_buffer
                            .push(data);
                    }
                }
            }
        }
    }
}
