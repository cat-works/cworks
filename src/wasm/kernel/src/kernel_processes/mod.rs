use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use crate::{fs::FSObj, rust_process::Session, SyscallData, SyscallError};

enum FSReturns {
    InvalidCommandFormat,
    UnsupportedMethod,
    InvalidHandle,
}

impl From<FSReturns> for String {
    fn from(value: FSReturns) -> Self {
        match value {
            FSReturns::InvalidCommandFormat => "InvalidCommandFormat".to_string(),
            FSReturns::UnsupportedMethod => "UnsupportedMethod".to_string(),
            FSReturns::InvalidHandle => "InvalidHandle".to_string(),
        }
    }
}

enum FSCommand {
    List,
}

impl TryFrom<String> for FSCommand {
    type Error = FSReturns;
    fn try_from(value: String) -> Result<Self, FSReturns> {
        match value.as_str() {
            "List" => Ok(FSCommand::List),
            _ => Err(FSReturns::InvalidCommandFormat),
        }
    }
}

struct FS {
    root: Arc<Mutex<FSObj>>,
}

impl FS {
    pub fn new(root: Arc<Mutex<FSObj>>) -> Self {
        Self { root }
    }

    pub fn cursor(&self) -> FSCursor {
        return FSCursor::new("/".to_string(), self.root.clone());
    }
}

impl Deref for FS {
    type Target = Arc<Mutex<FSObj>>;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl DerefMut for FS {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}

struct FSCursor {
    path: String,
    obj: Arc<Mutex<FSObj>>,
}

impl FSCursor {
    pub fn new(path: String, obj: Arc<Mutex<FSObj>>) -> Self {
        Self { path, obj }
    }

    pub fn list(&self) -> Result<Vec<String>, FSReturns> {
        match *self.obj.lock().unwrap() {
            FSObj::Dist(ref map) => Ok(map.keys().map(|x| x.to_string()).collect::<Vec<_>>()),
            _ => Err(FSReturns::UnsupportedMethod),
        }
    }
}

pub async fn fs_daemon_process(
    session: Arc<Session>,
    daemon: Arc<Mutex<FSObj>>,
) -> Result<i64, SyscallError> {
    let _s = session.ipc_create("system/file-system".to_string()).await?;

    let fs = FS::new(daemon.clone());
    let mut state = HashMap::new();

    loop {
        let data = session.get_syscall_data().await;
        match data {
            SyscallData::Connection { client, server: _ } => {
                state.insert(client.id, fs.cursor());
            }
            SyscallData::ReceivingData { focus, data } => {
                let r = match FSCommand::try_from(data.clone()) {
                    Ok(x) => x,
                    Err(e) => {
                        session.ipc_send(focus.clone(), e.into()).await?;
                        continue;
                    }
                };

                match r {
                    FSCommand::List => {
                        let state = match state.get(&focus.id) {
                            Some(x) => x,
                            None => {
                                session
                                    .ipc_send(focus.clone(), "InvalidHandle".to_string())
                                    .await?;
                                continue;
                            }
                        };
                        let ret = match state.list().map(|x| x.join(", ")) {
                            Ok(x) => x,
                            Err(e) => {
                                session.ipc_send(focus.clone(), e.into()).await?;
                                continue;
                            }
                        };

                        session.ipc_send(focus.clone(), ret).await?;
                    }
                }

                println!("Received: {} [Client -> Server]", data);

                break;
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
