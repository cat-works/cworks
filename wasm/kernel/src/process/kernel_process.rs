use std::collections::VecDeque;

use crate::obj_tree::FSObjRef;

use super::{Process, SyscallData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Sleeping(i64),   // TODO: Rename to WaitSeconds
    WaitingForEvent, // waiting for an channel event
}

pub struct KernelProcess {
    pub parent_pid: u64,
    pub process: Box<dyn Process>,
    pub outgoing_data_buffer: VecDeque<SyscallData>,
    pub status: ProcessStatus,
    pub waiters_pid: Vec<u64>, // PIDs of processes waiting for this process to finish
    pub listening_channels: Vec<FSObjRef>,
}

impl From<Box<dyn Process>> for KernelProcess {
    fn from(p: Box<dyn Process>) -> Self {
        Self {
            parent_pid: 0,
            process: p,
            outgoing_data_buffer: VecDeque::new(),
            status: ProcessStatus::Running,
            waiters_pid: vec![],
            listening_channels: Vec::new(),
        }
    }
}
