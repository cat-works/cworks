use crate::Handle;

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

impl Iterator for HandleIssuer {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(handle) = self.free_handles.pop() {
            Some(Handle::new(handle))
        } else {
            self.last_handle += 1;
            Some(Handle::new(self.last_handle))
        }
    }
}
