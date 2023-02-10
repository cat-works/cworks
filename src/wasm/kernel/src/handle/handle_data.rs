use std::sync::{Arc, Mutex};

use crate::{ipc::Ipc, Handle};

#[derive(Debug, Clone)]
pub enum HandleData {
    IpcServer {
        ipc: Arc<Mutex<Ipc>>,
    },
    IpcServerClient {
        server: Arc<Mutex<Ipc>>,
        client: Arc<Handle>,
    },
    IpcClient {
        server: Arc<Mutex<Ipc>>,
    },
    None,
}
