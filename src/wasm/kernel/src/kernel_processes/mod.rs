use std::{sync::{Arc, Mutex}, ops::{Deref, DerefMut}};

use crate::{fs::FSObj, rust_process::Session, SyscallData, SyscallError};

enum FSReturns {
    InvalidCommandFormat,
    UnsupportedMethod,
}

impl From<FSReturns> for String {
    fn from(value: FSReturns) -> Self {
        match value {
            FSReturns::InvalidCommandFormat => "InvalidCommandFormat".to_string(),
            FSReturns::UnsupportedMethod => "UnsupportedMethod".to_string(),
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


pub async fn fs_daemon_process(
    session: Arc<Session>,
    daemon: Arc<Mutex<FSObj>>,
) -> Result<i64, SyscallError> {
    let s = session.ipc_create("system/file-system".to_string()).await?;
    let mut sc = None;

    loop {
        let data = session.get_syscall_data().await;
        match data {
            SyscallData::Connection { client, server } => {
                // TODO: Handle this
            }
            SyscallData::ReceivingData { focus, data } if Option::Some(focus.clone()) == sc => {
                let r = match FSCommand::try_from(data) {
                    Ok(x) => x,
                    Err(e) => {
                        session.ipc_send(focus.clone(), e.into()).await?;
                        continue;
                    }
                };

                match r {
                    FSCommand::List => {
                        let daemon = daemon.lock().unwrap();
                        let list = match daemon {
                            FSObj::Dist(ref map) => {
                                map
                                    .keys()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                                    .clone()
                            },
                            _ => {
                                session
                                .ipc_send(focus.clone(), "UnsupportedMethod".to_string())
                                .await?;
                                continue;
                            }
                        }
                        session.ipc_send(focus.clone(), list).await?;
                        continue;
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
