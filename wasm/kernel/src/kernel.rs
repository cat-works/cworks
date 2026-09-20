use crate::{
    libs::{timestamp_ms, AutoMap},
    obj_tree::{initfs, FSObjRef, Object},
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
    pub fn register_process(&mut self, p: Box<dyn Process>) -> u64 {
        self.processes.add_value(p.into())
    }

    pub fn set_process_debug(&mut self, enabled: bool) {
        self.process_debug_enabled = enabled;
    }

    fn send_syscall_data(&mut self, pid: u64, data: SyscallData) {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.outgoing_data_buffer.push_back(data);
        } else {
            log::warn!("Process {pid} not found, failed to send syscall data: {data:?}");
        }
    }

    fn update_process_status(&mut self, pid: u64, status: ProcessStatus) {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.status = status;
        } else {
            log::warn!("Process {pid} not found, failed to update status to {status:?}");
        }
    }

    fn wait_process(&mut self, pid: u64, wait_target: u64) -> Result<(), SyscallData> {
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

    fn handle_syscall(&mut self, now: i64, pid: u64, res: PollResult) -> Result<(), SyscallData> {
        match res {
            PollResult::WaitForEvent => {
                let has_pending = !self
                    .processes
                    .get(&pid)
                    .unwrap()
                    .outgoing_data_buffer
                    .is_empty();
                if !has_pending {
                    self.update_process_status(pid, ProcessStatus::WaitingForEvent);
                }
            }
            PollResult::Pending => (),
            PollResult::Done => {
                for waiter in self.processes.get(&pid).unwrap().waiters_pid.clone() {
                    self.update_process_status(waiter, ProcessStatus::Running);
                }
                for obj in self
                    .processes
                    .get_mut(&pid)
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
                    callee_pid.retain(|&x| x != pid);
                    **obj.borrow_mut() = Object::Func { callee_pid };
                }
                self.processes.remove(&pid);
            }
            PollResult::Sleep(seconds) => {
                let duration_ms = (seconds * 1000.0) as i64;
                self.update_process_status(pid, ProcessStatus::Sleeping(now + duration_ms));
            }
            PollResult::WaitForProcess(waitee) => {
                self.wait_process(pid, waitee)?;
            }
            PollResult::Root => {
                self.send_syscall_data(pid, SyscallData::FSRoot(self.fs_root.clone()));
            }
            PollResult::Subscribe(func_obj) => {
                let mut callee_pid = {
                    let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                        return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                    };
                    callee_pid.clone()
                };
                callee_pid.push(pid);
                **func_obj.borrow_mut() = Object::Func { callee_pid };

                self.send_syscall_data(pid, SyscallData::FSSuccess);
                self.processes
                    .get_mut(&pid)
                    .unwrap()
                    .listening_channels
                    .push(func_obj);
            }
            PollResult::Unsubscribe(func_obj) => {
                let mut callee_pid = {
                    let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                        return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                    };
                    callee_pid.clone()
                };
                callee_pid.retain(|&x| x != pid);
                **func_obj.borrow_mut() = Object::Func { callee_pid };

                self.send_syscall_data(pid, SyscallData::FSSuccess);
            }
            PollResult::Publish(func_obj, content) => {
                let callee_pid = {
                    let Object::Func { ref callee_pid } = **func_obj.borrow() else {
                        return Err(SyscallData::Fail(SyscallError::InvalidRequest));
                    };
                    callee_pid.clone()
                };

                for callee_pid in callee_pid {
                    let syscall_data = SyscallData::Invoke {
                        caller_pid: pid,
                        obj: func_obj.clone(),
                        arg: content.clone(),
                    };
                    self.send_syscall_data(callee_pid, syscall_data);
                    self.update_process_status(callee_pid, ProcessStatus::Running);
                }

                self.send_syscall_data(pid, SyscallData::FSSuccess);
            }
            PollResult::GetPid => {
                self.send_syscall_data(pid, SyscallData::GetPid(pid));
            }
        }

        Ok(())
    }

    pub fn step(&mut self) {
        let now = timestamp_ms();
        let pid_list: Vec<u64> = self.processes.keys().copied().collect();

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

            if let Err(e) = self.handle_syscall(now, *pid, res) {
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
