pub(crate) mod fs;
pub(crate) mod initfs;
mod path;

use std::{collections::HashMap, sync::Arc};

use crate::{Session, SyscallData, SyscallError};

use self::fs::FSObj;

// TODO: (should) auto generate ts wrapper

enum FSReturns {
    InvalidCommandFormat,
    UnsupportedMethod,
    InvalidHandle,
    UnknownPath,
    UnknownError,
    ResourceIsBusy,
    Ok,
}

impl From<FSReturns> for String {
    fn from(value: FSReturns) -> Self {
        match value {
            FSReturns::InvalidCommandFormat => "InvalidCommandFormat".to_string(),
            FSReturns::UnsupportedMethod => "UnsupportedMethod".to_string(),
            FSReturns::InvalidHandle => "InvalidHandle".to_string(),
            FSReturns::UnknownPath => "UnknownPath".to_string(),
            FSReturns::UnknownError => "UnknownError".to_string(),
            FSReturns::ResourceIsBusy => "ResourceIsBusy".to_string(),
            FSReturns::Ok => "Ok".to_string(),
        }
    }
}

enum FSCommand {
    List,
    Cd(String),
    Get(String),
    Set(String, FSObj),
    Root,
}

impl TryFrom<String> for FSCommand {
    type Error = FSReturns;
    fn try_from(value: String) -> Result<Self, FSReturns> {
        if value.is_empty() {
            return Err(FSReturns::InvalidCommandFormat);
        }

        let tokens = value.split('?').collect::<Vec<_>>();
        match (tokens.len(), tokens[0]) {
            (1, "List") => Ok(FSCommand::List),
            (1, "Root") => Ok(FSCommand::Root),
            (2, "Cd") => Ok(FSCommand::Cd(tokens[1].to_string())),
            (2, "Get") => Ok(FSCommand::Get(tokens[1].to_string())),
            (3.., "Set") => Ok(FSCommand::Set(
                tokens[1].to_string(),
                FSObj::from_daemon_string(tokens[2..].join("?").to_string())?,
            )),
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

    pub fn exists(&self, path: String) -> bool {
        self.root.get_obj(path).is_ok()
    }

    pub fn list(&self, path: String) -> Result<Vec<String>, FSReturns> {
        match self
            .root
            .get_obj(path)
            .map_err(|_| FSReturns::UnknownPath)?
        {
            FSObj::Dict(x) => Ok(x
                .try_lock()
                .ok_or(FSReturns::ResourceIsBusy)?
                .keys()
                .cloned()
                .collect::<Vec<_>>()),
            _ => Err(FSReturns::UnsupportedMethod),
        }
    }

    pub fn get(&self, path: String) -> Result<FSObj, FSReturns> {
        self.root.get_obj(path).map_err(|_| FSReturns::UnknownPath)
    }
    pub fn set(&self, path: String, obj: FSObj) -> Result<(), FSReturns> {
        let p = self.root.get_obj(path::parent(&path).unwrap());
        if let Ok(FSObj::Dict(x)) = p {
            let mut x = x.try_lock().ok_or(FSReturns::ResourceIsBusy)?;
            x.insert(
                path::basename(&path)
                    .ok_or(FSReturns::UnknownPath)?
                    .to_string(),
                obj,
            );
            Ok(())
        } else {
            Err(FSReturns::UnsupportedMethod)
        }
    }
}

trait ToDaemonString {
    fn to_daemon_string(&self) -> Result<String, FSReturns>;
}

impl ToDaemonString for FSObj {
    fn to_daemon_string(&self) -> Result<String, FSReturns> {
        match self {
            FSObj::Boolean(x) => Ok(format!("Boolean?{}", x)),
            FSObj::Float(x) => Ok(format!("Float?{}", x)),
            FSObj::Int(x) => Ok(format!("Integer?{}", x)),
            FSObj::Double(x) => Ok(format!("Double?{}", x)),
            FSObj::String(x) => Ok(format!("String?{}", x)),
            FSObj::Bytes(x) => Ok(format!(
                "Bytes?{}",
                x.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("?")
            )),
            FSObj::Null => Ok("Null".to_string()),
            FSObj::Dict(_) => Err(FSReturns::UnsupportedMethod),
            FSObj::List(_) => Err(FSReturns::UnsupportedMethod),
            FSObj::Handle(_) => Err(FSReturns::UnsupportedMethod),
            FSObj::Dynamic(_) => Err(FSReturns::UnsupportedMethod),
        }
    }
}

trait FromDaemonString {
    fn from_daemon_string(s: String) -> Result<FSObj, FSReturns>;
}

impl FromDaemonString for FSObj {
    fn from_daemon_string(s: String) -> Result<FSObj, FSReturns> {
        let tokens = s.split('?').collect::<Vec<_>>();

        if tokens.len() < 2 {
            return Err(FSReturns::InvalidCommandFormat);
        }

        match tokens[0] {
            "Boolean" => Ok(FSObj::Boolean(
                tokens[1]
                    .parse::<bool>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?
                    .into(),
            )),
            "Float" => Ok(FSObj::Float(
                tokens[1]
                    .parse::<f32>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?
                    .into(),
            )),
            "Integer" => Ok(FSObj::Int(
                tokens[1]
                    .parse::<i128>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?
                    .into(),
            )),
            "Double" => Ok(FSObj::Double(
                tokens[1]
                    .parse::<f64>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?
                    .into(),
            )),
            "String" => Ok(FSObj::String(tokens[1].to_string().into())),
            "Bytes" => Ok(FSObj::Bytes(
                tokens[1..]
                    .iter()
                    .map(|x| x.parse::<u8>().map_err(|_| FSReturns::InvalidCommandFormat))
                    .collect::<Result<Vec<_>, FSReturns>>()?
                    .into(),
            )),
            "Null" => Ok(FSObj::Null),
            _ => Err(FSReturns::InvalidCommandFormat),
        }
    }
}

pub async fn fs_daemon_process(session: Arc<Session<FSObj>>) -> Result<i64, SyscallError> {
    log::debug!("FS: Starting Daemon");
    let _s = session.ipc_create("system/file-system".to_string()).await?;
    log::debug!("FS: IPC Created!");

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
                log::debug!("FS: Client[{}] <- [{}]", focus.id, data);
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
                        None => FSReturns::InvalidHandle.into(),
                    },
                    FSCommand::Root => {
                        state.insert(focus.id, "/".to_string());
                        FSReturns::Ok.into()
                    }
                    FSCommand::Cd(path) => match state
                        .get(&focus.id)
                        .map(|x| path::join(x, path.clone()))
                        .map(|x| (x.clone(), fs.exists(x)))
                    {
                        Some((p, f)) if f => {
                            state.insert(focus.id, p);
                            FSReturns::Ok.into()
                        }
                        Some((_, f)) if !f => FSReturns::UnknownPath.into(),
                        Some(_) => FSReturns::UnknownError.into(),
                        None => FSReturns::InvalidHandle.into(),
                    },
                    FSCommand::Get(s) => match state
                        .get(&focus.id)
                        .map(|x| fs.get(path::join(x, s)).map(|x| x.to_daemon_string()))
                    {
                        Some(Ok(Ok(x))) => x,
                        Some(Ok(Err(x))) => x.into(),
                        Some(Err(x)) => x.into(),
                        None => FSReturns::InvalidHandle.into(),
                    },
                    FSCommand::Set(s, obj) => {
                        match state.get(&focus.id).map(|x| fs.set(path::join(x, s), obj)) {
                            Some(Ok(())) => FSReturns::Ok.into(),
                            Some(Err(x)) => x.into(),
                            None => FSReturns::InvalidHandle.into(),
                        }
                    }
                };
                log::debug!("FS: Client[{}] -> {}", focus.id, ret);
                session.ipc_send(focus.clone(), ret).await?;
            }

            _ => {
                println!("{data:?}");
                panic!();
            }
        }
    }
}

// fn fs_daemon() -> Box<dyn Process> {
//     let fs_obj = ;
//
//     Box::new(RustProcess::new(&fs_daemon_process, fs_obj))
// }
