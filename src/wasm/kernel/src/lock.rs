use crate::pid::Pid;

#[derive(Debug, Hash)]
pub struct Lock {
    resource_path: String,
    kernel_side_lock_id: u128,
    managed_by: Pid,
    referenced_by: Vec<Pid>,
}

impl Lock {
    pub fn new(resource_path: String, managed_by: Pid) -> Lock {
        Lock {
            managed_by,
            resource_path,
            kernel_side_lock_id: 0,
            referenced_by: vec![],
        }
    }
}
