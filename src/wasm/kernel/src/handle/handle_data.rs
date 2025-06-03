use std::{cell::RefCell, rc::Rc};

use crate::{ipc::Ipc, Handle};

#[derive(Debug, Clone, Default)]
pub enum HandleData {
    IpcServer {
        ipc: Rc<RefCell<Ipc>>,
    },
    IpcServerClient {
        server: Rc<RefCell<Ipc>>,
        client: Handle,
    },
    IpcClient {
        server: Rc<RefCell<Ipc>>,
    },
    #[default]
    None,
}

impl std::fmt::Display for HandleData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleData::IpcClient { server: _ } => {
                // let x = server.borrow();
                // write!(f, "IpcClient({x})")?
            }
            HandleData::IpcServer { ipc: _ } => {
                // let x = ipc.borrow();
                // write!(f, "IpcServer({x})")?
            }
            HandleData::IpcServerClient {
                server: _,
                client: _,
            } => {
                // let x = server.borrow();
                // write!(f, "IpcServerClient({x}:{client})")?
            }
            HandleData::None => write!(f, "None")?,
        };
        Ok(())
    }
}
