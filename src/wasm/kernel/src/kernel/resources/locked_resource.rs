use super::KernelResource;

#[derive(Debug, Hash)]
pub struct LockedResource {
    resource_path: String,
    kernel_side_lock_id: u128,
    managed_by: PID,
}

impl LockedResource {
    pub fn new(resource_path: String) -> LockedResource {
        LockedResource {
            resource_path,
            kernel_side_lock_id: 0,
            pid: 0,
        }
    }
    pub fn get_resource_path(&self) -> &KernelResource {
        &self.resource_path
    }

    pub fn get_kernel_side_lock_id(&self) -> u128 {
        self.kernel_side_lock_id
    }

    pub fn set_kernel_side_lock_id(&mut self, kernel_side_lock_id: u128) {
        self.kernel_side_lock_id = kernel_side_lock_id;
    }

    pub fn get_pid(&self) -> u128 {
        self.pid
    }
}
