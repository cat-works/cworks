use crate::kernel::pid::PID;

#[derive(Debug, Hash)]
pub struct Lock {
    resource_path: String,
    kernel_side_lock_id: u128,
    managed_by: PID,
    referenced_by: Vec<PID>,
}

impl Lock {
    pub fn new(resource_path: String, managed_by: PID) -> Lock {
        Lock {
            managed_by,
            resource_path,
            kernel_side_lock_id: 0,
            referenced_by: vec![],
        }
    }
    pub fn get_resource_path(&self) -> &String {
        &self.resource_path
    }

    pub fn get_kernel_side_lock_id(&self) -> u128 {
        self.kernel_side_lock_id
    }

    pub fn get_managed_by(&self) -> &PID {
        &self.managed_by
    }

    pub fn get_referenced_by(&self) -> &Vec<PID> {
        &self.referenced_by
    }
}
