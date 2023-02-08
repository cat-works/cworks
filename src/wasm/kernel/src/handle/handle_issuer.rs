use std::collections::HashMap;

use crate::Handle;

#[derive(Debug)]
pub enum HandleData {
    IpcServer(String),
    IpcClient(String),
}

pub struct HandleIssuer {
    handle_map: HashMap<Handle, HandleData>,
    last_handle: u128,
    free_handles: Vec<u128>,
}

impl Default for HandleIssuer {
    fn default() -> Self {
        HandleIssuer {
            handle_map: HashMap::default(),
            last_handle: 0,
            free_handles: vec![],
        }
    }
}

impl HandleIssuer {
    pub fn get_new_handle(&mut self, pid: u128, data: HandleData) -> Handle {
        let handle = if let Some(handle) = self.free_handles.pop() {
            Handle::new(pid, handle)
        } else {
            self.last_handle += 1;
            Handle::new(pid, self.last_handle)
        };

        self.handle_map.insert(handle.clone(), data);
        handle
    }
}
