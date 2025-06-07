use std::{cell::RefCell, collections::HashMap, rc::Rc};

use log::info;

use crate::{
    fs::{fs_daemon_process, initfs},
    handle::{HandleData, HandleIssuer},
    ipc::Ipc,
    libs::{timestamp_ms, AutoMap},
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
    ipc_instances: HashMap<String, Rc<RefCell<Ipc>>>,
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

        ret.processes.add_value(KernelProcess {
            parent_pid: 0,
            process: Box::new(RustProcess::new(&fs_daemon_process, initfs())),
            outgoing_data_buffer: vec![],
            status: ProcessStatus::Running,
        });

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
        let now = timestamp_ms();
        for (pid, p) in &mut self.processes.iter_mut() {
            // log::trace!("Polling {pid} {:?}", p.status);
            if let ProcessStatus::Sleeping(t) = p.status {
                if t >= now {
                    continue;
                } else {
                    /* debug!("Waking up Process<{pid}> ({now:6.4} <= {t:6.4})"); */
                    p.status = ProcessStatus::Running;
                }
            }

            let data = p.outgoing_data_buffer.pop().unwrap_or(SyscallData::None);

            let res = p.process.poll(&data);
            // log::trace!("{pid}: {res:?}");
            match res {
                PollResult::Pending => (),
                PollResult::Done(n) => {
                    log::debug!("Process<{pid}> Returns {n}");
                    self.actions.push(KernelAction::ProcessKill(*pid));
                }
                PollResult::Syscall(s) => match s {
                    Syscall::Sleep(seconds) => {
                        let duration_ms = (seconds * 1000.0) as i64;
                        p.status = ProcessStatus::Sleeping(now + duration_ms);
                        /* log::debug!(
                            "Process<{pid}> Sleeps for {seconds:6.4} seconds since {now:6.4}"
                        ); */
                    }
                    Syscall::IpcCreate(ref name) => {
                        if self.ipc_instances.contains_key(name) {
                            p.outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::AlreadyExists));
                            continue;
                        }
                        // TODO: Authority Check
                        let ipc = Rc::new(RefCell::new(Ipc::default()));

                        let handle = self
                            .handle_issuer
                            .get_new_handle(*pid, HandleData::IpcServer { ipc: ipc.clone() });
                        ipc.borrow_mut().set_server_handle(handle.clone());

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
                            let mut ipc = ipc.borrow_mut();

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
                            let ipc = server.borrow_mut();
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
                    self.processes.remove(&pid);
                }
                KernelAction::SendSyscallData(pid, data) => {
                    let process = self.processes.get_mut(&pid);
                    match process {
                        Some(process) => {
                            process.outgoing_data_buffer.push(data);
                        }
                        None => {
                            log::warn!("Process {pid} not found! (ignored)");
                        }
                    }
                }
            }
        }
    }
    pub fn start(&mut self) {
        while !self.processes.is_empty() {
            self.step();
        }
    }
}
