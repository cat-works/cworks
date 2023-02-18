mod path;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{fs::FSObj, rust_process::Session, SyscallData, SyscallError};

enum FSReturns {
    InvalidCommandFormat,
    UnsupportedMethod,
    InvalidHandle,
    UnknownPath,
    Ok,
}

impl From<FSReturns> for String {
    fn from(value: FSReturns) -> Self {
        match value {
            FSReturns::InvalidCommandFormat => "InvalidCommandFormat".to_string(),
            FSReturns::UnsupportedMethod => "UnsupportedMethod".to_string(),
            FSReturns::InvalidHandle => "InvalidHandle".to_string(),
            FSReturns::UnknownPath => "UnknownPath".to_string(),
            FSReturns::Ok => "Ok".to_string(),
        }
    }
}

enum FSCommand {
    List,
    Cd(String),
}

impl TryFrom<String> for FSCommand {
    type Error = FSReturns;
    fn try_from(value: String) -> Result<Self, FSReturns> {
        if value == "" {
            return Err(FSReturns::InvalidCommandFormat);
        }

        let toks = value.split("?").collect::<Vec<_>>();
        match (toks.len(), toks[0]) {
            (1, "List") => Ok(FSCommand::List),
            (2, "Cd") => Ok(FSCommand::Cd(toks[1].to_string())),
            _ => Err(FSReturns::InvalidCommandFormat),
        }
    }
}

struct FS {
    pub root: FSObj,
}

impl FS {
    pub fn new(root: FSObj) -> Self {
        Self { root }
    }
    pub(crate) fn get_obj(&self) -> &FSObj {
        &self.root
    }
    pub(crate) fn get_obj_mut(&mut self) -> &mut FSObj {
        &mut self.root
    }

    pub fn exists(&self, path: String) -> bool {
        self.root.get_obj(path).is_ok()
    }

    pub fn list(&self, path: String) -> Result<Vec<String>, FSReturns> {
        match self
            .root
            .get_obj(path)
            .map_err(|_| FSReturns::UnknownPath)?
        {
            FSObj::Dist(x) => Ok(x.keys().map(|x| x.clone()).collect::<Vec<_>>()),
            _ => Err(FSReturns::UnsupportedMethod),
        }
    }
}

pub async fn fs_daemon_process(session: Arc<Session<FSObj>>) -> Result<i64, SyscallError> {
    let _s = session.ipc_create("system/file-system".to_string()).await?;

    let fs = session.get_value();
    let fs = FS::new(fs);

    let mut state = HashMap::new();

    loop {
        let data = session.get_syscall_data().await;
        match data {
            SyscallData::Connection { client, server: _ } => {
                state.insert(client.id, "/".to_string());
            }
            SyscallData::ReceivingData { focus, data } => {
                println!("Received: Client[{}] -> Server: {}", focus.id, data);
                let r = match FSCommand::try_from(data.clone()) {
                    Ok(x) => x,
                    Err(e) => {
                        session.ipc_send(focus.clone(), e.into()).await?;
                        continue;
                    }
                };

                let ret = match r {
                    FSCommand::List => match state.get(&focus.id).map(|x| fs.list(x.to_string())) {
                        Some(Ok(x)) => x.join("?"),
                        Some(Err(e)) => e.into(),
                        None => "InvalidHandle".to_string(),
                    },
                    FSCommand::Cd(path) => match state
                        .get(&focus.id)
                        .map(|x| path::join(x, path.clone()))
                        .map(|x| (x.clone(), fs.exists(x)))
                    {
                        Some((p, f)) if f == true => {
                            state.insert(focus.id, p);
                            "Success".to_string()
                        }
                        Some((_, f)) if f == false => "UnknownPath".to_string(),
                        Some(_) => "UnknownError".to_string(),
                        None => "InvalidHandle".to_string(),
                    },
                };
                println!("Received: {} [Client -> Server]", data);
                session.ipc_send(focus.clone(), ret).await?;
            }

            _ => {
                println!("{data:?}");
                panic!();
            }
        }
    }

    Ok(0i64)
}

// fn fs_daemon() -> Box<dyn Process> {
//     let fs_obj = ;
//
//     Box::new(RustProcess::new(&fs_daemon_process, fs_obj))
// }
