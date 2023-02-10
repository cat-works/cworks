use std::{collections::HashMap, sync::Arc};

use crate::{
    fs::{FSObj, RefOrVal},
    handle::HandleData,
    Handle,
};

#[derive(Debug)]
pub struct IpcMessage {
    pub from: Option<Arc<Handle>>,
    pub message: String,
}

impl From<IpcMessage> for FSObj {
    fn from(x: IpcMessage) -> FSObj {
        let mut message = HashMap::new();
        message.insert("handle".to_string(), x.from.into());
        message.insert(
            "message".to_string(),
            FSObj::String(RefOrVal::Ref(Box::new(x.message))),
        );
        FSObj::Dist(RefOrVal::Ref(Box::new(message)))
    }
}

#[derive(Debug)]
pub struct Ipc {
    buffer: Vec<IpcMessage>,
    server: Option<Arc<Handle>>,
    clients: Vec<Arc<Handle>>,
}

impl Ipc {
    pub fn new() -> Ipc {
        Ipc {
            buffer: vec![],
            clients: vec![],
            server: None,
        }
    }
    pub fn connect(&mut self, client: Arc<Handle>) {
        self.clients.push(client);
    }

    pub fn set_server_handle(&mut self, server: Arc<Handle>) {
        self.server = Some(server);
    }

    pub fn get_server_handle(&self) -> &Option<Arc<Handle>> {
        &self.server
    }

    pub fn send(&mut self, data: String, client: Option<Arc<Handle>>) {
        self.buffer.push(IpcMessage {
            from: client,
            message: data,
        })
    }
}

impl From<Ipc> for FSObj {
    fn from(x: Ipc) -> FSObj {
        let mut root = HashMap::new();
        root.insert("server".to_string(), x.server.into());

        let mut clients = vec![];
        for client in x.clients {
            clients.push(FSObj::Handle(RefOrVal::Ref(Box::new(client))));
        }
        root.insert("clients".to_string(), FSObj::List(RefOrVal::Val(clients)));

        let mut buffer = vec![];
        for x in x.buffer {
            buffer.push(x.into());
        }
        root.insert("buffer".to_string(), FSObj::List(RefOrVal::Val(buffer)));

        FSObj::Dist(RefOrVal::Val(root))
    }
}
