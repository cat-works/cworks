use std::{collections::HashMap, sync::Arc};

use crate::{fs::FSObj, handle::HandleData, Handle};

#[derive(Debug)]
pub struct IpcMessage {
    pub from: Option<Handle>,
    pub message: String,
}

impl From<IpcMessage> for FSObj {
    fn from(x: IpcMessage) -> FSObj {
        let mut message = HashMap::new();
        message.insert("handle".to_string(), x.from.into());
        message.insert("message".to_string(), FSObj::String(Arc::new(x.message)));
        FSObj::Dict(Arc::new(message.into()))
    }
}

#[derive(Debug, Default)]
pub struct Ipc {
    server: Option<Handle>,
    clients: Vec<Handle>,
}
impl std::fmt::Display for Ipc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IPC({} < [{}])",
            self.server.as_ref().unwrap(),
            self.clients
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Ipc {
    pub fn connect(&mut self, client: Handle) {
        self.clients.push(client);
    }

    pub fn get_server_side_handle(&self, required_client: Handle) -> Option<Handle> {
        for client in &self.clients {
            match &client.data {
                HandleData::IpcServerClient {
                    server: _,
                    client: c,
                } if c.id == required_client.id => {
                    return Some(client.clone());
                }
                _ => (),
            }
        }
        None
    }

    pub fn set_server_handle(&mut self, server: Handle) {
        self.server = Some(server);
    }

    pub fn get_server_handle(&self) -> &Option<Handle> {
        &self.server
    }

    pub fn send(&mut self, data: String, client: Option<Handle>) -> (u128, IpcMessage) {
        (
            self.server.clone().unwrap().pid,
            IpcMessage {
                from: client,
                message: data,
            },
        )
    }
}

impl From<Ipc> for FSObj {
    fn from(x: Ipc) -> FSObj {
        let mut root = HashMap::new();
        root.insert("server".to_string(), x.server.into());

        let mut clients = vec![];
        for client in x.clients {
            clients.push(FSObj::Handle(client));
        }
        root.insert("clients".to_string(), FSObj::List(Arc::new(clients)));

        FSObj::Dict(Arc::new(root.into()))
    }
}
