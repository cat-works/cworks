use crate::Handle;

use super::HandleData;

#[derive(Default)]
pub struct HandleIssuer {
    last_handle: u128,
    free_handles: Vec<u128>,
}

impl HandleIssuer {
    pub fn get_new_handle(&mut self, pid: u128, data: HandleData) -> Handle {
        if let Some(handle) = self.free_handles.pop() {
            Handle::new(pid, handle, data)
        } else {
            self.last_handle += 1;
            Handle::new(pid, self.last_handle, data)
        }
    }
}
