use std::{thread::sleep, time::Duration};

use crate::{
    libs::{split_filename, timestamp_ms, AutoMap},
    obj_tree::{initfs, FSFrontend, FSObjRef, Object},
    process::{ProcessStatus, SyscallData, SyscallError},
};

use super::process::{KernelProcess, PollResult, Process};

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    fs_root: FSObjRef,

    process_debug_enabled: bool,
}

impl Default for Kernel {
    fn default() -> Self {
        Self {
            processes: AutoMap::new(1),
            fs_root: initfs(),

            process_debug_enabled: false,
        }
    }
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) -> u128 {
        self.processes.add_value(p.into())
    }

    pub fn set_process_debug(&mut self, enabled: bool) {
        self.process_debug_enabled = enabled;
    }

    fn send_syscall_data(&mut self, pid: u128, data: SyscallData) {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.outgoing_data_buffer.push_back(data);
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

    fn wait_process(&mut self, pid: u128, wait_target: u128) -> Result<(), SyscallData> {
        if !self.processes.contains_key(&wait_target) {
            return Err(SyscallData::Fail(SyscallError::NoSuchEntry));
        }
        self.update_process_status(pid, ProcessStatus::WaitingForEvent);

        self.processes
            .get_mut(&wait_target)
            .unwrap()
            .waiters_pid
            .push(pid);

        Ok(())
    }

    fn handle_syscall(&mut self, now: i64, pid: &u128, res: PollResult) -> Result<(), SyscallData> {
        let fs_frontend = FSFrontend::new(self.fs_root.clone());

        match res {
            PollResult::WaitForEvent => {
                let has_pending = !self
                    .processes
                    .get(pid)
                    .unwrap()
                    .outgoing_data_buffer
                    .is_empty();
                if !has_pending {
                    self.update_process_status(*pid, ProcessStatus::WaitingForEvent);
                }
            }
            PollResult::Pending => (),
            PollResult::Done => {
                for waiter in self.processes.get(pid).unwrap().waiters_pid.clone() {
                    self.update_process_status(waiter, ProcessStatus::Running);
                }
                for obj in self
                    .processes
                    .get_mut(pid)
                    .unwrap()
                    .listening_channels
                    .drain(..)
                {
                    let mut callee_pid = {
                        let Object::Func { ref callee_pid } = **obj.borrow() else {
                            return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                        };
                        callee_pid.clone()
                    };
                    callee_pid.retain(|&x| x != *pid);
                    **obj.borrow_mut() = Object::Func { callee_pid };
                }
                self.processes.remove(pid);
            }
            PollResult::Sleep(seconds) => {
                let duration_ms = (seconds * 1000.0) as i64;
                self.update_process_status(*pid, ProcessStatus::Sleeping(now + duration_ms));
            }
            PollResult::WaitForProcess(waitee) => {
                self.wait_process(*pid, waitee)?;
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
                self.processes
                    .get_mut(pid)
                    .unwrap()
                    .listening_channels
                    .push(func_obj);
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
                .pop_front()
                .unwrap_or(SyscallData::None);

            if self.process_debug_enabled && !matches!(data, SyscallData::None) {
                log::trace!("Process<{pid}> <-- {data:?}");
            }
            let res = self.processes.get_mut(pid).unwrap().process.poll(&data);
            if self.process_debug_enabled && !matches!(res, PollResult::Pending) {
                log::trace!("Process<{pid}> --> {res:?}");
            }

            if let Err(e) = self.handle_syscall(now, pid, res) {
                log::warn!("Process<{pid}> syscall failed: {e:?}");
                self.send_syscall_data(*pid, e);
            }
        }

        let is_any_process_running = self.processes.values().any(|p| {
            p.status == ProcessStatus::Running || matches!(p.status, ProcessStatus::Sleeping(_))
        });
        if !is_any_process_running {
            log::debug!("No running processes, kernel exiting");
            self.processes.clear();
        }
    }
    pub fn start(&mut self) {
        while !self.processes.is_empty() {
            self.step();
        }
    }
}
