use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    WakeUp(u128),
}

struct PWaitingPair {
    waitee: u128,
    waiter: u128,
}

pub struct Kernel {
    processes: RefCell<AutoMap<RefCell<KernelProcess>>>,
    ipc_instances: RefCell<HashMap<String, Rc<RefCell<Ipc>>>>,
    handle_issuer: HandleIssuer,
    waiting_pairs: RefCell<HashMap<u128, Vec<PWaitingPair>>>,
}

impl Default for Kernel {
    fn default() -> Kernel {
        let mut ret = Kernel {
            processes: RefCell::new(AutoMap::new()),
            ipc_instances: RefCell::new(HashMap::new()),
            handle_issuer: HandleIssuer::default(),
            waiting_pairs: RefCell::new(HashMap::new()),
        };

        ret.processes
            .borrow_mut()
            .add_value(RefCell::new(KernelProcess {
                parent_pid: 0,
                process: Box::new(RustProcess::new(&fs_daemon_process, initfs())),
                outgoing_data_buffer: vec![],
                status: ProcessStatus::Running,
            }));

        ret
    }
}

impl Kernel {
    pub fn get_ipc_names(&self) -> Vec<String> {
        self.ipc_instances.borrow().keys().cloned().collect()
    }

    pub fn register_process(&self, p: Box<dyn Process>) {
        self.processes
            .borrow_mut()
            .add_value(RefCell::new(p.into()));
    }

    pub fn step(&self) {
        let mut actions = vec![];

        let now = timestamp_ms();
        let process_keys: Vec<u128> = self.processes.borrow().keys().cloned().collect();

        for (pid, p) in self.processes.borrow().iter() {
            // log::trace!("Polling {pid} {:?}", p.status);
            if let ProcessStatus::Sleeping(t) = p.borrow().status {
                if t >= now {
                    continue;
                } else {
                    /* debug!("Waking up Process<{pid}> ({now:6.4} <= {t:6.4})"); */
                    actions.push(KernelAction::WakeUp(*pid));
                }
            }

            if p.borrow().status != ProcessStatus::Running {
                continue;
            }

            let data = p
                .borrow_mut()
                .outgoing_data_buffer
                .pop()
                .unwrap_or(SyscallData::None);

            let res = p.borrow_mut().process.poll(&data);
            // log::trace!("{pid}: {res:?}");

            match res {
                PollResult::Pending => (),
                PollResult::Done(n) => {
                    log::debug!("Process<{pid}> Returns {n}");
                    actions.push(KernelAction::ProcessKill(*pid));

                    // Lookup for waiting processes
                    let pairs = self.waiting_pairs.borrow_mut().remove(pid);
                    if let Some(pairs) = pairs {
                        for pair in pairs {
                            actions.push(KernelAction::WakeUp(pair.waiter));
                        }
                    }
                }
                PollResult::Syscall(s) => {
                    match s {
                        Syscall::Sleep(seconds) => {
                            let duration_ms = (seconds * 1000.0) as i64;
                            p.borrow_mut().status = ProcessStatus::Sleeping(now + duration_ms);
                            /* log::debug!(
                                "Process<{pid}> Sleeps for {seconds:6.4} seconds since {now:6.4}"
                            ); */
                        }
                        Syscall::IpcCreate(ref name) => {
                            if self.ipc_instances.borrow().contains_key(name) {
                                p.borrow_mut()
                                    .outgoing_data_buffer
                                    .push(SyscallData::Fail(SyscallError::AlreadyExists));
                                continue;
                            }
                            // TODO: Authority Check
                            let ipc = Rc::new(RefCell::new(Ipc::default()));

                            let handle = self
                                .handle_issuer
                                .get_new_handle(*pid, HandleData::IpcServer { ipc: ipc.clone() });
                            ipc.borrow_mut().set_server_handle(handle.clone());

                            self.ipc_instances
                                .borrow_mut()
                                .insert(name.clone(), ipc.clone());

                            p.borrow_mut()
                                .outgoing_data_buffer
                                .push(SyscallData::Handle(handle));
                            continue;
                        }
                        Syscall::IpcConnect(ref name) => {
                            if !self.ipc_instances.borrow().contains_key(name) {
                                p.borrow_mut()
                                    .outgoing_data_buffer
                                    .push(SyscallData::Fail(SyscallError::NoSuchEntry));
                                continue;
                            }

                            let ipc = self.ipc_instances.borrow().get(name).unwrap().clone();

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
                                actions.push(KernelAction::SendSyscallData(
                                    server.pid,
                                    SyscallData::Connection {
                                        client: server_client_handle.clone(),
                                        server: server.clone(),
                                    },
                                ));
                            }

                            p.borrow_mut()
                                .outgoing_data_buffer
                                .push(SyscallData::Handle(client_handle));
                            continue;
                        }
                        Syscall::Send(ref handle, ref data) => match handle.data {
                            HandleData::IpcServer { ipc: _ } => {
                                p.borrow_mut()
                                    .outgoing_data_buffer
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

                                actions.push(act);
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

                                actions.push(act);
                                continue;
                            }
                            _ => {
                                p.borrow_mut()
                                    .outgoing_data_buffer
                                    .push(SyscallData::Fail(SyscallError::UnknownHandle));
                                continue;
                            }
                        },
                        Syscall::WaitForProcess(waitee) => {
                            if process_keys.contains(&waitee) {
                                p.borrow_mut().status = ProcessStatus::WaitingForProcess;

                                let pair = PWaitingPair {
                                    waitee,
                                    waiter: *pid,
                                };
                                if let Some(waiters) =
                                    self.waiting_pairs.borrow_mut().get_mut(&waitee)
                                {
                                    waiters.push(pair);
                                } else {
                                    self.waiting_pairs.borrow_mut().insert(waitee, vec![pair]);
                                }
                            } else {
                                p.borrow_mut()
                                    .outgoing_data_buffer
                                    .push(SyscallData::Fail(SyscallError::NoSuchEntry));
                            }
                        }
                    }
                }
            }
        }

        for act in actions {
            match act {
                KernelAction::ProcessKill(pid) => {
                    self.processes.borrow_mut().remove(&pid);
                }
                KernelAction::SendSyscallData(pid, data) => {
                    let mut processes = self.processes.borrow_mut();
                    let process = processes.get_mut(&pid);
                    match process {
                        Some(process) => {
                            process.borrow_mut().outgoing_data_buffer.push(data);
                        }
                        None => {
                            log::warn!("Process {pid} not found! (ignored)");
                        }
                    }
                }
                KernelAction::WakeUp(pid) => {
                    let mut processes = self.processes.borrow_mut();
                    let process = processes.get_mut(&pid);

                    if let Some(process) = process {
                        process.borrow_mut().status = ProcessStatus::Running;
                    } else {
                        log::warn!("Process {pid} not found for wake up! (ignored)");
                    }
                }
            }
        }
    }
    pub fn start(&mut self) {
        while !self.processes.borrow().is_empty() {
            self.step();
        }
    }
}
