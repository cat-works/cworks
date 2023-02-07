use std::collections::HashMap;

use crate::{
    fs::{FSObj, RefOrVal},
    Handle,
};

pub struct IpcMessage {
    pub from: Handle,
    pub message: String,
}

impl From<IpcMessage> for FSObj {
    fn from(x: IpcMessage) -> FSObj {
        let mut message = HashMap::new();
        message.insert(
            "handle".to_string(),
            FSObj::Handle(RefOrVal::Ref(Box::new(x.from))),
        );
        message.insert(
            "message".to_string(),
            FSObj::String(RefOrVal::Ref(Box::new(x.message))),
        );
        FSObj::Dist(RefOrVal::Ref(Box::new(message)))
    }
}

pub struct Ipc {
    buffer: Vec<IpcMessage>,
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
    pub fn connect(&mut self, client: Handle) {
        self.clients.push(client);
    }
}

impl From<Ipc> for FSObj {
    fn from(x: Ipc) -> FSObj {
        let mut root = HashMap::new();
        root.insert(
            "server".to_string(),
            FSObj::Handle(RefOrVal::Ref(Box::new(x.server))),
        );

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
