use super::HandleData;
use crate::Handle;

pub type HandleRef = u128;

#[derive(Default)]
pub struct HandleIssuer {
    last_handle: u128,
    free_handles: Vec<u128>,
    handle_table: std::collections::HashMap<u128, Handle>,
}

impl HandleIssuer {
    pub fn get_new_handle(&mut self, pid: u128, data: HandleData) -> HandleRef {
        let hid = if let Some(handle) = self.free_handles.pop() {
            handle
        } else {
            let last_handle = self.last_handle;
            let handle = last_handle.checked_add(1).expect("Handle overflow");
            self.last_handle = handle;
            handle
        };

        self.handle_table.insert(hid, Handle::new(pid, hid, data));
        hid
    }

    pub fn get_handle(&self, handle: HandleRef) -> Option<Handle> {
        self.handle_table.get(&handle).cloned()
    }

    pub fn get_handle_data(&self, handle: HandleRef) -> Option<(u128, HandleData)> {
        let handle = self.get_handle(handle)?;
        Some((handle.pid, handle.data.clone()))
    }
}
