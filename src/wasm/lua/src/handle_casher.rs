use std::collections::HashMap;

use kernel::Handle;

#[derive(Default)]
pub struct HandleCasher {
    handles: HashMap<u128, Handle>,
}

impl HandleCasher {
    pub fn register_handle(&mut self, h: Handle) {
        self.handles.insert(h.id, h);
    }

    pub fn get_handle(&self, id: u128) -> Option<Handle> {
        self.handles.get(&id).cloned()
    }
}
