use crate::Handle;

pub struct IPC {
    buffer: Vec<(u128, String)>,
    server: Handle,
    clients: Vec<Handle>,
}

impl IPC {
    pub fn new(server: Handle) -> IPC {
        IPC {
            buffer: vec![],
            clients: vec![],
            server,
        }
    }
}
