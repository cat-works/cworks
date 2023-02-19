use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    handle::{HandleData, HandleIssuer},
    initfs::initfs,
    ipc::Ipc,
    kernel_processes::fs_daemon_process,
    libs::{timestamp, AutoMap},
    process::{ProcessStatus, Syscall, SyscallData, SyscallError},
    RustProcess,
};

use super::process::{KernelProcess, PollResult, Process};

enum KernelAction {
    ProcessKill(u128),
    SendSyscallData(u128, SyscallData),
}

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    ipc_instances: HashMap<String, Arc<Mutex<Ipc>>>,
    handle_issuer: HandleIssuer,
    actions: Vec<KernelAction>,
}

impl Default for Kernel {
    fn default() -> Kernel {
        let mut ret = Kernel {
            processes: AutoMap::new(),
            ipc_instances: HashMap::new(),
            handle_issuer: HandleIssuer::default(),
            actions: vec![],
        };

        ret.register_process(Box::new(RustProcess::new(&fs_daemon_process, initfs())));

        ret
    }
}

impl Kernel {
    pub fn get_ipc_names(&self) -> Vec<String> {
        self.ipc_instances.keys().cloned().collect()
    }

    pub fn register_process(&mut self, p: Box<dyn Process>) {
        self.processes.add_value(p.into());
    }

    fn step_all_processes(&mut self) {
        for (pid, p) in &mut self.processes.map {
            // println!("Polling {pid} {:?}", p.status);
            if let ProcessStatus::Sleeping(t) = p.status {
                if t > timestamp() {
                    continue;
                } else {
                    p.status = ProcessStatus::Running;
                }
            }

            let data = p.outgoing_data_buffer.pop().unwrap_or(SyscallData::None);

            let res = p.process.poll(&data);
            // println!("{pid}: {res:?}");
            match res {
                PollResult::Pending => (),
                PollResult::Done(n) => {
                    println!("Process<{pid}> Returns {n}");
                    self.actions.push(KernelAction::ProcessKill(*pid));
                }
                PollResult::Syscall(s) => match s {
                    Syscall::Sleep(seconds) => {
                        p.status = ProcessStatus::Sleeping(timestamp() + seconds);
                    }
                    Syscall::IpcCreate(ref name) => {
                        if self.ipc_instances.contains_key(name) {
                            p.outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::AlreadyExists));
                            continue;
                        }

                        let ipc = Arc::new(Mutex::new(Ipc::default()));

                        let handle = self
                            .handle_issuer
                            .get_new_handle(*pid, HandleData::IpcServer { ipc: ipc.clone() });
                        ipc.lock().unwrap().set_server_handle(handle.clone());

                        self.ipc_instances.insert(name.clone(), ipc.clone());

                        p.outgoing_data_buffer.push(SyscallData::Handle(handle));
                        continue;
                    }
                    Syscall::IpcConnect(ref name) => {
                        if !self.ipc_instances.contains_key(name) {
                            p.outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::NoSuchEntry));
                            continue;
                        }

                        let ipc = self.ipc_instances.get(name).unwrap();

                        let client_handle = self.handle_issuer.get_new_handle(
                            *pid,
                            HandleData::IpcClient {
                                server: ipc.clone(),
                            },
                        );

                        let server_client_handle = self.handle_issuer.get_new_handle(
                            *pid,
                            HandleData::IpcServerClient {
                                server: ipc.clone(),
                                client: client_handle.clone(),
                            },
                        );

                        {
                            let mut ipc = ipc.lock().unwrap();

                            ipc.connect(server_client_handle.clone());
                            let server = ipc.get_server_handle().as_ref().unwrap();
                            self.actions.push(KernelAction::SendSyscallData(
                                server.pid,
                                SyscallData::Connection {
                                    client: server_client_handle.clone(),
                                    server: server.clone(),
                                },
                            ));
                        }

                        p.outgoing_data_buffer
                            .push(SyscallData::Handle(client_handle));
                        continue;
                    }
                    Syscall::Send(ref handle, ref data) => match handle.data {
                        HandleData::IpcServer { ipc: _ } => {
                            p.outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::UnknownHandle));
                            continue;
                        }
                        HandleData::IpcClient { ref server } => {
                            let mut ipc = server.lock().unwrap();
                            let (pid, _) = ipc.send(data.clone(), Some(handle.clone()));

                            let handle = ipc.get_server_side_handle(handle.clone());
                            let act = KernelAction::SendSyscallData(
                                pid,
                                SyscallData::ReceivingData {
                                    focus: handle.unwrap(),
                                    data: data.to_string(),
                                },
                            );

                            self.actions.push(act);
                        }
                        HandleData::IpcServerClient {
                            server: _,
                            ref client,
                        } => {
                            let act = KernelAction::SendSyscallData(
                                client.pid,
                                SyscallData::ReceivingData {
                                    focus: client.clone(),
                                    data: data.to_string(),
                                },
                            );

                            self.actions.push(act);
                            continue;
                        }
                        _ => {
                            p.outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::UnknownHandle));
                            continue;
                        }
                    },
                },
            }
        }
    }
    pub fn step(&mut self) {
        self.actions = vec![];

        self.step_all_processes();

        while let Some(act) = self.actions.pop() {
            match act {
                KernelAction::ProcessKill(pid) => {
                    self.processes.map.remove(&pid);
                }
                KernelAction::SendSyscallData(pid, data) => {
                    let process = self.processes.map.get_mut(&pid);
                    match process {
                        Some(process) => {
                            process.outgoing_data_buffer.push(data);
                        }
                        None => {
                            println!("Process {pid} not found! (ignored)");
                        }
                    }
                }
            }
        }
    }
    pub fn start(&mut self) {
        while !self.processes.map.is_empty() {
            self.step();
        }
    }
}
