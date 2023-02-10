use std::sync::Arc;

use crate::Handle;

use super::HandleData;

pub struct HandleIssuer {
    last_handle: u128,
    free_handles: Vec<u128>,
}

impl Default for HandleIssuer {
    fn default() -> Self {
        HandleIssuer {
            last_handle: 0,
            free_handles: vec![],
        }
    }
}

impl HandleIssuer {
    pub fn get_new_handle(&mut self, pid: u128, data: HandleData) -> Arc<Handle> {
        let handle = if let Some(handle) = self.free_handles.pop() {
            Handle::new(pid, handle, data)
        } else {
            self.last_handle += 1;
            Handle::new(pid, self.last_handle, data)
        };

        Arc::new(handle)
    }
}
