use std::sync::{Arc, Mutex};

use crate::{ipc::Ipc, Handle};

#[derive(Debug, Clone)]
pub enum HandleData {
    IpcServer {
        ipc: Arc<Mutex<Ipc>>,
    },
    IpcServerClient {
        server: Arc<Mutex<Ipc>>,
        client: Handle,
    },
    IpcClient {
        server: Arc<Mutex<Ipc>>,
    },
    None,
}

impl std::fmt::Display for HandleData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleData::IpcClient { server } => {
                let lock = server.try_lock();
                match lock {
                    Ok(x) => write!(f, "IpcClient({})", x)?,
                    Err(_) => write!(f, "IpcClient")?,
                }
            }
            HandleData::IpcServer { ipc } => {
                let lock = ipc.try_lock();
                match lock {
                    Ok(x) => write!(f, "IpcServer({})", x)?,
                    Err(_) => write!(f, "IpcServer")?,
                }
            }
            HandleData::IpcServerClient { server, client } => {
                let lock = server.try_lock();
                match lock {
                    Ok(x) => write!(f, "IpcServerClient({}:{client})", x)?,
                    Err(_) => write!(f, "IpcServerClient(<Server>:{client})")?,
                }
            }
            HandleData::None => write!(f, "None")?,
        };
        Ok(())
    }
}
