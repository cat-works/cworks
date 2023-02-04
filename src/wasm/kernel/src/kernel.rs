use crate::process::{Handle, Syscall, SyscallData, SyscallError};

use super::{
    automap::AutoMap,
    // fs::FSObj,
    process::{KernelProcess, PollResult, Process},
    resources::{KernelResource, LockedResource},
};

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    // fs_root: FSObj,
    locks: AutoMap<LockedResource>,
}

impl Default for Kernel {
    fn default() -> Kernel {
        Kernel {
            processes: AutoMap::new(),
            locks: AutoMap::new(),
            // fs_root: FSObj::Dist(HashMap::new()),
        }
    }
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) {
        self.processes.add_value(p.into());
    }

    pub fn acquire_lock(&mut self, resource: KernelResource) {
        self.locks.add_value(LockedResource::new(resource));
    }

    pub fn start(&mut self) {
        while !self.processes.map.is_empty() {
            let mut process_to_kill = vec![];
            for (pid, p) in &mut self.processes.map {
                match p.process.poll(&p.system_call_returns) {
                    PollResult::Pending => (),
                    PollResult::Done(n) => {
                        println!("Process<{pid}> Returns {n}");
                        process_to_kill.push(*pid);
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
                                // TODO: IPC Create
                                p.system_call_returns =
                                    SyscallData::Handle(Err(SyscallError::NotImplemented));
                            }
                            Syscall::IPC_Connect(ref name) => {
                                // TODO: IPC Connect
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
