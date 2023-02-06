use std::collections::HashMap;

use crate::{
    fs::RefOrVal,
    handle::HandleIssuer,
    ipc::Ipc,
    libs::{timestamp, AutoMap},
    process::{ProcessStatus, Syscall, SyscallData, SyscallError},
};

use super::{
    fs::FSObj,
    process::{KernelProcess, PollResult, Process},
};

pub struct Kernel {
    processes: AutoMap<KernelProcess>,
    fs_root: FSObj,
    ipc_instances: HashMap<String, Ipc>,
    handle_issuer: HandleIssuer,
}

impl Default for Kernel {
    fn default() -> Kernel {
        Kernel {
            processes: AutoMap::new(),
            fs_root: FSObj::Dist(RefOrVal::Val(HashMap::new())),
            ipc_instances: HashMap::new(),
            handle_issuer: HandleIssuer::default(),
        }
    }
}

impl Kernel {
    pub fn register_process(&mut self, p: Box<dyn Process>) {
        self.processes.add_value(p.into());
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
                    }
                    PollResult::Syscall(s) => {
                        match s {
                            Syscall::IPC_Create(ref name) => {
                                if self.ipc_instances.contains_key(name) {
                                    p.system_call_returns =
                                        SyscallData::Handle(Err(SyscallError::AlreadyExists));
                                    break;
                                }

                                let name = format!("/sys/ipc/{name}");
                                let handle = self.handle_issuer.next().unwrap();

                                let ipc = Ipc::new(handle.clone());
                                self.ipc_instances.insert(name.clone(), ipc);

                                *self.fs_root.get_obj_mut(name, true).unwrap() = FSObj::from(ipc);
                                p.system_call_returns = SyscallData::Handle(Ok(handle));
                                break;
                            }
                            Syscall::IPC_Connect(ref name) => {
                                //let name = format!("/sys/ipc/{name}");
                                //let handle_id = match self.locks.map.get(name) {
                                //    None => {
                                //        p.system_call_returns =
                                //            SyscallData::Handle(Err(SyscallError::NoSuchEntry));
                                //        break;
                                //    }
                                //    Some(x) => match *x.get_resource() {
                                //        KernelResource::Object(x) => x,
                                //    },
                                //};
                                //
                                //let ipc = IPC::new(Handle::new(handle_id));
                                //
                                //p.system_call_returns =
                                //    SyscallData::Handle(Ok(Handle { id: handle_id }));
                                //break;
                                //
                                //let obj =
                                //    self.fs_root.get_obj_mut(format!("/sys/ipc/{name}"), false);
                                //if let Err(_) = obj {
                                //    p.system_call_returns =
                                //        SyscallData::Handle(Err(SyscallError::UnreachableEntry));
                                //    break;
                                //}
                                //let fsobj = obj.unwrap();
                                //
                                //if let FSObj::Handle(handle_id) = fsobj {
                                //    p.system_call_returns =
                                //        SyscallData::Handle(Ok(Handle { id: *handle_id }));
                                //} else {
                                //    p.system_call_returns =
                                //        SyscallData::Handle(Err(SyscallError::NoSuchEntry));
                                //}
                                //break;
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
