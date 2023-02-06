use crate::Handle;

pub struct Ipc {
    buffer: Vec<(u128, String)>,
    server: Handle,
    clients: Vec<Handle>,
}

impl Ipc {
    pub fn new(server: Handle) -> Ipc {
        Ipc {
            buffer: vec![],
            clients: vec![],
            server,
        }
    }
}
