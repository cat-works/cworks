use std::{cell::RefCell, collections::HashMap};

use crate::{
    libs::{split_filename, timestamp_ms, AutoMap},
    obj_tree::{initfs, FSFrontend, FSObjRef, FSReturns, Object},
    process::{ProcessStatus, Syscall, SyscallData, SyscallError},
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
    waiting_pairs: RefCell<HashMap<u128, Vec<PWaitingPair>>>,
    fs_root: FSObjRef,
}

impl Default for Kernel {
    fn default() -> Kernel {
        Kernel {
            processes: RefCell::new(AutoMap::new()),
            waiting_pairs: RefCell::new(HashMap::new()),
            fs_root: initfs(),
        }
    }
}

impl Kernel {
    pub fn register_process(&self, p: Box<dyn Process>) {
        self.processes
            .borrow_mut()
            .add_value(RefCell::new(p.into()));
    }

    pub fn step(&mut self) {
        let mut actions = vec![];

        let now = timestamp_ms();
        let process_keys: Vec<u128> = self.processes.borrow().keys().cloned().collect();

        for (pid, p) in self.processes.borrow().iter() {
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

            if !matches!(data, SyscallData::None) {
                log::debug!("Process<{pid}> <-- {:?}", data)
            };
            let res = p.borrow_mut().process.poll(&data);
            if !matches!(res, PollResult::Pending) {
                log::debug!("Process<{pid}> --> {:?}", res);
            }

            match res {
                PollResult::Pending => (),
                PollResult::Done(n) => {
                    log::debug!("Process<{pid}> ==> {n}");
                    actions.push(KernelAction::ProcessKill(*pid));

                    // Lookup for waiting processes
                    let pairs = self.waiting_pairs.borrow_mut().remove(pid);
                    if let Some(pairs) = pairs {
                        for pair in pairs {
                            if pair.waitee != *pid {
                                continue;
                            }
                            actions.push(KernelAction::WakeUp(pair.waiter));
                        }
                    }
                }
                PollResult::Syscall(s) => {
                    let fs_frontend = FSFrontend::new(self.fs_root.clone());
                    match s {
                        Syscall::Sleep(seconds) => {
                            let duration_ms = (seconds * 1000.0) as i64;
                            p.borrow_mut().status = ProcessStatus::Sleeping(now + duration_ms);
                            /* log::debug!(
                                "Process<{pid}> Sleeps for {seconds:6.4} seconds since {now:6.4}"
                            ); */
                        }
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
                        Syscall::List(path) => {
                            let res = fs_frontend.list(path);
                            p.borrow_mut().outgoing_data_buffer.push(
                                res.map(SyscallData::FSList)
                                    .unwrap_or_else(SyscallData::FSError),
                            );
                        }
                        Syscall::Stat(path) => {
                            let stat = fs_frontend.stat(path);
                            p.borrow_mut().outgoing_data_buffer.push(
                                stat.map(SyscallData::FSStat)
                                    .unwrap_or_else(SyscallData::FSError),
                            );
                        }
                        Syscall::Get(path) => {
                            let res = fs_frontend.get(path);
                            p.borrow_mut().outgoing_data_buffer.push(
                                res.map(SyscallData::FSGet)
                                    .unwrap_or_else(SyscallData::FSError),
                            );
                        }
                        Syscall::Set(path, obj) => {
                            let res = fs_frontend.set(path, obj);
                            p.borrow_mut().outgoing_data_buffer.push(
                                res.map(|_| SyscallData::FSSuccess)
                                    .unwrap_or_else(SyscallData::FSError),
                            );
                        }
                        Syscall::Mkdir(path, name) => {
                            let res = fs_frontend.mkdir(path, name);
                            p.borrow_mut().outgoing_data_buffer.push(
                                res.map(|_| SyscallData::FSSuccess)
                                    .unwrap_or_else(SyscallData::FSError),
                            );
                        }
                        Syscall::Subscribe(path) => {
                            let (dir, fname) = match split_filename(path)
                                .ok_or(SyscallData::FSError(FSReturns::InvalidCommandFormat))
                                .and_then(|(dir, fname)| {
                                    self.fs_root
                                        .follow(dir)
                                        .map_err(SyscallData::FSError)
                                        .map(|d| (d, fname))
                                }) {
                                Ok(a) => a,
                                Err(e) => {
                                    p.borrow_mut().outgoing_data_buffer.push(e);
                                    continue;
                                }
                            };

                            let func_obj = match dir
                                .get_obj(fname.clone())
                                .map_err(SyscallData::FSError)
                                .or_else(|_| {
                                    let obj: FSObjRef = Object::Func { callee_pid: vec![] }.into();
                                    dir.add_child(fname.clone(), obj.clone())
                                        .map(|_| obj)
                                        .map_err(SyscallData::FSError)
                                }) {
                                Ok(obj) => obj,
                                Err(e) => {
                                    p.borrow_mut().outgoing_data_buffer.push(e);
                                    continue;
                                }
                            };
                            let mut callee_pid = {
                                let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                                    p.borrow_mut()
                                        .outgoing_data_buffer
                                        .push(SyscallData::FSError(FSReturns::UnsupportedMethod));
                                    continue;
                                };
                                callee_pid.clone()
                            };
                            callee_pid.push(*pid);
                            **func_obj.borrow_mut() = Object::Func { callee_pid };

                            p.borrow_mut()
                                .outgoing_data_buffer
                                .push(SyscallData::FSSuccess);
                        }
                        Syscall::Unsubscribe(path) => {
                            let (dir, fname) = match split_filename(path)
                                .ok_or(SyscallData::FSError(FSReturns::InvalidCommandFormat))
                                .and_then(|(dir, fname)| {
                                    self.fs_root
                                        .follow(dir)
                                        .map_err(SyscallData::FSError)
                                        .map(|d| (d, fname))
                                }) {
                                Ok(a) => a,
                                Err(e) => {
                                    p.borrow_mut().outgoing_data_buffer.push(e);
                                    continue;
                                }
                            };

                            let func_obj = match dir
                                .get_obj(fname.clone())
                                .map_err(SyscallData::FSError)
                                .or_else(|_| {
                                    let obj: FSObjRef = Object::Func { callee_pid: vec![] }.into();
                                    dir.add_child(fname.clone(), obj.clone())
                                        .map(|_| obj)
                                        .map_err(SyscallData::FSError)
                                }) {
                                Ok(obj) => obj,
                                Err(e) => {
                                    p.borrow_mut().outgoing_data_buffer.push(e);
                                    continue;
                                }
                            };
                            let mut callee_pid = {
                                let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                                    p.borrow_mut()
                                        .outgoing_data_buffer
                                        .push(SyscallData::FSError(FSReturns::UnsupportedMethod));
                                    continue;
                                };
                                callee_pid.clone()
                            };
                            callee_pid.retain(|&x| x != *pid);
                            **func_obj.borrow_mut() = Object::Func { callee_pid };

                            p.borrow_mut()
                                .outgoing_data_buffer
                                .push(SyscallData::FSSuccess);
                        }
                        Syscall::Publish(path, content) => {
                            let func_obj = match self.fs_root.follow(path.clone()) {
                                Ok(obj) => obj,
                                Err(e) => {
                                    p.borrow_mut()
                                        .outgoing_data_buffer
                                        .push(SyscallData::FSError(e));
                                    continue;
                                }
                            };

                            let callee_pid = {
                                let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                                    p.borrow_mut()
                                        .outgoing_data_buffer
                                        .push(SyscallData::FSError(FSReturns::UnsupportedMethod));
                                    continue;
                                };
                                callee_pid.clone()
                            };

                            for pid in callee_pid {
                                actions.push(KernelAction::SendSyscallData(
                                    pid,
                                    SyscallData::Invoke {
                                        caller_pid: pid,
                                        path: path.clone(),
                                        arg: content.clone(),
                                    },
                                ));
                            }

                            p.borrow_mut()
                                .outgoing_data_buffer
                                .push(SyscallData::FSSuccess);
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
