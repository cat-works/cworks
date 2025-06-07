use std::cell::RefCell;

use crate::Handle;

use super::HandleData;

// TODO: Handle recycling
// 1. Process exits, handle is freed

#[derive(Default)]
pub struct HandleIssuer {
    last_handle: RefCell<u128>,
    free_handles: RefCell<Vec<u128>>,
}

impl HandleIssuer {
    pub fn get_new_handle(&self, pid: u128, data: HandleData) -> Handle {
        if let Some(handle) = self.free_handles.borrow_mut().pop() {
            Handle::new(pid, handle, data)
        } else {
            let last_handle = *self.last_handle.borrow();
            let handle = last_handle.checked_add(1).expect("Handle overflow");

            *self.last_handle.borrow_mut() = handle;
            Handle::new(pid, handle, data)
        }
    }
}
