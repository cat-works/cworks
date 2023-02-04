use std::collections::HashMap;

use crate::process::{Handle, ProcessStatus, Syscall, SyscallData, SyscallError};

use super::{
    automap::AutoMap,
    fs::FSObj,
    process::{KernelProcess, PollResult, Process},
    resources::{KernelResource, LockedResource},
};

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    fs_root: FSObj,
    locks: AutoMap<LockedResource>,
}

impl Default for Kernel {
    fn default() -> Kernel {
        Kernel {
            processes: AutoMap::new(),
            locks: AutoMap::new(),
            fs_root: FSObj::Dist(HashMap::new()),
        }
    }
}

fn timestamp() -> f32 {
    (chrono::Utc::now().timestamp_millis() - 1672498800000) as f32 / 1000.0
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) {
        self.processes.add_value(p.into());
    }

    pub fn acquire_lock(&mut self, resource: KernelResource) -> u128 {
        self.locks.add_value(LockedResource::new(resource))
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
                match p.process.poll(&p.system_call_returns) {
                    PollResult::Pending => (),
                    PollResult::Done(n) => {
                        println!("Process<{pid}> Returns {n}");
                        process_to_kill.push(*pid);
                    }
                    PollResult::Sleep(n) => {
                        p.status = ProcessStatus::Sleeping(timestamp() + n);
                        println!("Process<{pid}> Sleeps for {n}ms");
                    }
                    PollResult::Syscall(s) => {
                        match s {
                            Syscall::Lock(ref path) => {
                                // Lock Check
                                for lock in self.locks.map.values() {
                                    let KernelResource::Object(ref path2) = lock.get_resource();
                                    if path.starts_with(path2) {
                                        p.system_call_returns =
                                            SyscallData::Handle(Err(SyscallError::AlreadyExists));
                                        break;
                                    }
                                }

                                let res =
                                    LockedResource::new(KernelResource::Object(path.to_string()));
                                let key = self.locks.add_value(res);
                                p.system_call_returns = SyscallData::Handle(Ok(Handle::new(key)));
                            }
                            Syscall::IPC_Create(ref name) => {
                                let obj =
                                    self.fs_root.get_obj_mut(format!("/sys/ipc/{name}"), true);
                                if let Err(_) = obj {
                                    p.system_call_returns =
                                        SyscallData::Handle(Err(SyscallError::UnreachableEntry));
                                    break;
                                }
                                let fsobj = obj.unwrap();

                                let obj = KernelResource::Object(name.to_string());
                                let handle_id = self.locks.add_value(LockedResource::new(obj));

                                *fsobj = FSObj::Handle(handle_id);

                                p.system_call_returns =
                                    SyscallData::Handle(Ok(Handle { id: handle_id }));
                                break;
                            }
                            Syscall::IPC_Connect(ref name) => {
                                let obj =
                                    self.fs_root.get_obj_mut(format!("/sys/ipc/{name}"), false);
                                if let Err(_) = obj {
                                    p.system_call_returns =
                                        SyscallData::Handle(Err(SyscallError::UnreachableEntry));
                                    break;
                                }
                                let fsobj = obj.unwrap();

                                if let FSObj::Handle(handle_id) = fsobj {
                                    p.system_call_returns =
                                        SyscallData::Handle(Ok(Handle { id: *handle_id }));
                                } else {
                                    p.system_call_returns =
                                        SyscallData::Handle(Err(SyscallError::NoSuchEntry));
                                }
                                break;
                            }
                            Syscall::Send(ref handle, ref data) => {
                                // TODO: IPC Send
                                p.system_call_returns =
                                    SyscallData::Handle(Err(SyscallError::NotImplemented));
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
