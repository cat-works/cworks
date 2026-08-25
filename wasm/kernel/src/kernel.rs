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

    fn send_syscall_data(&mut self, pid: u128, data: SyscallData) {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.outgoing_data_buffer.push(data);
        } else {
            log::warn!("Process {pid} not found, failed to send syscall data: {data:?}");
        }
    }

    fn update_process_status(&mut self, pid: u128, status: ProcessStatus) {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.status = status;
        } else {
            log::warn!("Process {pid} not found, failed to update status to {status:?}");
        }
    }

    fn handle_syscall(&mut self, now: i64, pid: &u128, res: PollResult) -> Result<(), SyscallData> {
        let fs_frontend = FSFrontend::new(self.fs_root.clone());

        match res {
            PollResult::WaitForEvent => {
                self.update_process_status(*pid, ProcessStatus::WaitingForEvent);
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
                        self.update_process_status(pair.waiter, ProcessStatus::Running);
                    }
                }
            }
            PollResult::Sleep(seconds) => {
                let duration_ms = (seconds * 1000.0) as i64;
                self.update_process_status(*pid, ProcessStatus::Sleeping(now + duration_ms));
            }
            PollResult::WaitForProcess(waitee) => {
                if !self.processes.contains_key(&waitee) {
                    return Err(SyscallData::Fail(SyscallError::NoSuchEntry));
                }
                self.update_process_status(*pid, ProcessStatus::WaitingForProcess);

                let pair = PWaitingPair {
                    waitee,
                    waiter: *pid,
                };
                if let Some(waiters) = self.waiting_pairs.get_mut(&waitee) {
                    waiters.push(pair);
                } else {
                    self.waiting_pairs.insert(waitee, vec![pair]);
                }
            }
            PollResult::List(path) => {
                let res = fs_frontend.list(&path);
                self.send_syscall_data(
                    *pid,
                    res.map_or_else(SyscallData::Fail, SyscallData::FSList),
                );
            }
            PollResult::Stat(path) => {
                let stat = fs_frontend.stat(&path);
                self.send_syscall_data(
                    *pid,
                    stat.map_or_else(SyscallData::Fail, SyscallData::FSStat),
                );
            }
            PollResult::Get(path) => {
                let res = fs_frontend.get(&path);
                self.send_syscall_data(
                    *pid,
                    res.map_or_else(SyscallData::Fail, SyscallData::FSGet),
                );
            }
            PollResult::Set(path, obj) => {
                let res = fs_frontend.set(&path, &obj);
                self.send_syscall_data(
                    *pid,
                    res.map_or_else(SyscallData::Fail, |()| SyscallData::FSSuccess),
                );
            }
            PollResult::Mkdir(path, name) => {
                let res = fs_frontend.mkdir(&path, &name);
                self.send_syscall_data(
                    *pid,
                    res.map_or_else(SyscallData::Fail, |()| SyscallData::FSSuccess),
                );
            }
            PollResult::Subscribe(path) => {
                let (dir, fname) = split_filename(&path)
                    .ok_or(SyscallData::Fail(SyscallError::InvalidRequest))
                    .and_then(|(dir, fname)| {
                        self.fs_root
                            .follow(&dir)
                            .map_err(SyscallData::Fail)
                            .map(|d| (d, fname))
                    })?;

                let func_obj = dir
                    .get_obj(&fname)
                    .map_err(SyscallData::Fail)
                    .or_else(|_| {
                        let obj: FSObjRef = Object::Func { callee_pid: vec![] }.into();
                        dir.add_child(&fname, &obj)
                            .map(|()| obj)
                            .map_err(SyscallData::Fail)
                    })?;
                let mut callee_pid = {
                    let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                        return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                    };
                    callee_pid.clone()
                };
                callee_pid.push(*pid);
                **func_obj.borrow_mut() = Object::Func { callee_pid };

                self.send_syscall_data(*pid, SyscallData::FSSuccess);
            }
            PollResult::Unsubscribe(path) => {
                let func_obj = self.fs_root.follow(&path).map_err(SyscallData::Fail)?;

                let mut callee_pid = {
                    let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                        return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                    };
                    callee_pid.clone()
                };
                callee_pid.retain(|&x| x != *pid);
                **func_obj.borrow_mut() = Object::Func { callee_pid };

                self.send_syscall_data(*pid, SyscallData::FSSuccess);
            }
            PollResult::Publish(path, content) => {
                let func_obj = self.fs_root.follow(&path).map_err(SyscallData::Fail)?;

                let callee_pid = {
                    let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                        return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                    };
                    callee_pid.clone()
                };

                for pid in callee_pid {
                    let syscall_data = SyscallData::Invoke {
                        caller_pid: pid,
                        path: path.clone(),
                        arg: content.clone(),
                    };
                    self.send_syscall_data(pid, syscall_data);
                    self.update_process_status(pid, ProcessStatus::Running);
                }

                self.send_syscall_data(*pid, SyscallData::FSSuccess);
            }
            PollResult::GetPid => {
                self.send_syscall_data(*pid, SyscallData::GetPid(*pid));
            }
        }

        Ok(())
    }

    pub fn step(&mut self) {
        let now = timestamp_ms();
        let pid_list: Vec<u128> = self.processes.keys().copied().collect();

        for pid in &pid_list {
            if let ProcessStatus::Sleeping(t) = self.processes.get(pid).unwrap().status {
                if t >= now {
                    continue;
                }
                self.update_process_status(*pid, ProcessStatus::Running);
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

            if let Err(e) = self.handle_syscall(now, pid, res) {
                log::warn!("Process<{pid}> syscall failed: {e:?}");
                self.send_syscall_data(*pid, e);
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
