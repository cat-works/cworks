use std::collections::HashMap;

use crate::{
    libs::{split_filename, timestamp_ms, AutoMap},
    obj_tree::{initfs, FSFrontend, FSObjRef, Object},
    process::{ProcessStatus, SyscallData, SyscallError},
};

use super::process::{KernelProcess, PollResult, Process};

struct PWaitingPair {
    waitee: u128,
    waiter: u128,
}

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    waiting_pairs: HashMap<u128, Vec<PWaitingPair>>,
    fs_root: FSObjRef,
}

impl Default for Kernel {
    fn default() -> Self {
        Self {
            processes: AutoMap::new(1),
            waiting_pairs: HashMap::new(),
            fs_root: initfs(),
        }
    }
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) -> u128 {
        self.processes.add_value(p.into())
    }

    pub fn step(&mut self) {
        let now = timestamp_ms();
        let pid_list: Vec<u128> = self.processes.keys().copied().collect();

        for pid in &pid_list {
            if let ProcessStatus::Sleeping(t) = self.processes.get(pid).unwrap().status {
                if t >= now {
                    continue;
                }
                self.processes.get_mut(pid).unwrap().status = ProcessStatus::Running;
            }

            if self.processes.get(pid).unwrap().status != ProcessStatus::Running {
                continue;
            }

            let data = self
                .processes
                .get_mut(pid)
                .unwrap()
                .outgoing_data_buffer
                .pop()
                .unwrap_or(SyscallData::None);

            if !matches!(data, SyscallData::None) {
                log::trace!("Process<{pid}> <-- {data:?}");
            }
            let res = self.processes.get_mut(pid).unwrap().process.poll(&data);
            if !matches!(res, PollResult::Pending) {
                log::trace!("Process<{pid}> --> {res:?}");
            }

            let fs_frontend = FSFrontend::new(self.fs_root.clone());

            match res {
                PollResult::WaitForEvent => {
                    self.processes.get_mut(pid).unwrap().status = ProcessStatus::WaitingForEvent;
                }
                PollResult::Pending => (),
                PollResult::Done => {
                    self.processes.remove(pid);

                    let pairs = self.waiting_pairs.remove(pid);
                    if let Some(pairs) = pairs {
                        for pair in pairs {
                            if pair.waitee != *pid {
                                continue;
                            }
                            if let Some(process) = self.processes.get_mut(&pair.waiter) {
                                process.status = ProcessStatus::Running;
                            } else {
                                log::warn!(
                                    "Process {} is waiting for {} but it does not exist",
                                    pair.waiter,
                                    pair.waitee
                                );
                            }
                        }
                    }
                }
                PollResult::Sleep(seconds) => {
                    let duration_ms = (seconds * 1000.0) as i64;
                    self.processes.get_mut(pid).unwrap().status =
                        ProcessStatus::Sleeping(now + duration_ms);
                    /* log::debug!(
                        "Process<{pid}> Sleeps for {seconds:6.4} seconds since {now:6.4}"
                    ); */
                }
                PollResult::WaitForProcess(waitee) => {
                    if pid_list.contains(&waitee) {
                        self.processes.get_mut(pid).unwrap().status =
                            ProcessStatus::WaitingForProcess;

                        let pair = PWaitingPair {
                            waitee,
                            waiter: *pid,
                        };
                        if let Some(waiters) = self.waiting_pairs.get_mut(&waitee) {
                            waiters.push(pair);
                        } else {
                            self.waiting_pairs.insert(waitee, vec![pair]);
                        }
                    } else {
                        self.processes
                            .get_mut(pid)
                            .unwrap()
                            .outgoing_data_buffer
                            .push(SyscallData::Fail(SyscallError::NoSuchEntry));
                    }
                }
                PollResult::List(path) => {
                    let res = fs_frontend.list(&path);
                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(res.map_or_else(SyscallData::Fail, SyscallData::FSList));
                }
                PollResult::Stat(path) => {
                    let stat = fs_frontend.stat(&path);
                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(stat.map_or_else(SyscallData::Fail, SyscallData::FSStat));
                }
                PollResult::Get(path) => {
                    let res = fs_frontend.get(&path);
                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(res.map_or_else(SyscallData::Fail, SyscallData::FSGet));
                }
                PollResult::Set(path, obj) => {
                    let res = fs_frontend.set(&path, &obj);
                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(res.map_or_else(SyscallData::Fail, |()| SyscallData::FSSuccess));
                }
                PollResult::Mkdir(path, name) => {
                    let res = fs_frontend.mkdir(&path, &name);
                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(res.map_or_else(SyscallData::Fail, |()| SyscallData::FSSuccess));
                }
                PollResult::Subscribe(path) => {
                    let (dir, fname) = match split_filename(&path)
                        .ok_or(SyscallData::Fail(SyscallError::InvalidRequest))
                        .and_then(|(dir, fname)| {
                            self.fs_root
                                .follow(&dir)
                                .map_err(SyscallData::Fail)
                                .map(|d| (d, fname))
                        }) {
                        Ok(a) => a,
                        Err(e) => {
                            self.processes
                                .get_mut(pid)
                                .unwrap()
                                .outgoing_data_buffer
                                .push(e);
                            continue;
                        }
                    };

                    let func_obj =
                        match dir.get_obj(&fname).map_err(SyscallData::Fail).or_else(|_| {
                            let obj: FSObjRef = Object::Func { callee_pid: vec![] }.into();
                            dir.add_child(&fname, &obj)
                                .map(|()| obj)
                                .map_err(SyscallData::Fail)
                        }) {
                            Ok(obj) => obj,
                            Err(e) => {
                                self.processes
                                    .get_mut(pid)
                                    .unwrap()
                                    .outgoing_data_buffer
                                    .push(e);
                                continue;
                            }
                        };
                    let mut callee_pid = {
                        let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                            self.processes
                                .get_mut(pid)
                                .unwrap()
                                .outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::InvalidRequest));
                            continue;
                        };
                        callee_pid.clone()
                    };
                    callee_pid.push(*pid);
                    **func_obj.borrow_mut() = Object::Func { callee_pid };

                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(SyscallData::FSSuccess);
                }
                PollResult::Unsubscribe(path) => {
                    let (dir, fname) = match split_filename(&path)
                        .ok_or(SyscallData::Fail(SyscallError::InvalidRequest))
                        .and_then(|(dir, fname)| {
                            self.fs_root
                                .follow(&dir)
                                .map_err(SyscallData::Fail)
                                .map(|d| (d, fname))
                        }) {
                        Ok(a) => a,
                        Err(e) => {
                            self.processes
                                .get_mut(pid)
                                .unwrap()
                                .outgoing_data_buffer
                                .push(e);
                            continue;
                        }
                    };

                    let func_obj =
                        match dir.get_obj(&fname).map_err(SyscallData::Fail).or_else(|_| {
                            let obj: FSObjRef = Object::Func { callee_pid: vec![] }.into();
                            dir.add_child(&fname, &obj.clone())
                                .map(|()| obj)
                                .map_err(SyscallData::Fail)
                        }) {
                            Ok(obj) => obj,
                            Err(e) => {
                                self.processes
                                    .get_mut(pid)
                                    .unwrap()
                                    .outgoing_data_buffer
                                    .push(e);
                                continue;
                            }
                        };
                    let mut callee_pid = {
                        let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                            self.processes
                                .get_mut(pid)
                                .unwrap()
                                .outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::InvalidRequest));
                            continue;
                        };
                        callee_pid.clone()
                    };
                    callee_pid.retain(|&x| x != *pid);
                    **func_obj.borrow_mut() = Object::Func { callee_pid };

                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(SyscallData::FSSuccess);
                }
                PollResult::Publish(path, content) => {
                    let func_obj = match self.fs_root.follow(&path) {
                        Ok(obj) => obj,
                        Err(e) => {
                            self.processes
                                .get_mut(pid)
                                .unwrap()
                                .outgoing_data_buffer
                                .push(SyscallData::Fail(e));
                            continue;
                        }
                    };

                    let callee_pid = {
                        let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                            self.processes
                                .get_mut(pid)
                                .unwrap()
                                .outgoing_data_buffer
                                .push(SyscallData::Fail(SyscallError::InvalidRequest));
                            continue;
                        };
                        callee_pid.clone()
                    };

                    for pid in callee_pid {
                        let syscall_data = SyscallData::Invoke {
                            caller_pid: pid,
                            path: path.clone(),
                            arg: content.clone(),
                        };
                        self.processes
                            .get_mut(&pid)
                            .map(|p| p.outgoing_data_buffer.push(syscall_data))
                            .unwrap_or_else(|| {
                                log::warn!("Process {pid} not found! (ignored)");
                            });
                    }

                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(SyscallData::FSSuccess);
                }
                PollResult::GetPid => {
                    self.processes
                        .get_mut(pid)
                        .unwrap()
                        .outgoing_data_buffer
                        .push(SyscallData::GetPid(*pid));
                }
            }
        }

        let is_any_process_running =
            self.processes
                .values()
                .map(|p| p.status.clone())
                .any(|status| {
                    status == ProcessStatus::Running || matches!(status, ProcessStatus::Sleeping(_))
                });
        if !is_any_process_running {
            self.processes.clear();
        }
    }
    pub fn start(&mut self) {
        while !self.processes.is_empty() {
            self.step();
        }
    }
}
