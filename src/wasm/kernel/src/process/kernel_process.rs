use super::{Process, SyscallData};

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Running,
    Sleeping(f32), // TODO: Rename to WaitSeconds
                   // TODO: WaitIPCCreate(String)
}

pub struct KernelProcess {
    pub parent_pid: u128,
    pub process: Box<dyn Process>,
    pub outgoing_data_buffer: Vec<SyscallData>,
    pub status: ProcessStatus,
}

impl From<Box<dyn Process>> for KernelProcess {
    fn from(p: Box<dyn Process>) -> Self {
        KernelProcess {
            parent_pid: 0,
            process: p,
            outgoing_data_buffer: vec![],
            status: ProcessStatus::Running,
        }
    }
}
