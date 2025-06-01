use crate::{handle::HandleData, Handle};

#[derive(Debug)]
pub struct IpcMessage {
    pub from: Option<Handle>,
    pub message: String,
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

    pub fn send(&self, data: String, client: Option<Handle>) -> (u128, IpcMessage) {
        (
            self.server.as_ref().unwrap().pid,
            IpcMessage {
                from: client,
                message: data,
            },
        )
    }
}
