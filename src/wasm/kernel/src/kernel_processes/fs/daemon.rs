use std::ops::Deref;

use crate::{
    fs::{
        traits::{DaemonCommunicable, DaemonString},
        FSCommand, FSFrontend, FSReturns,
    },
    Handle, RustProcessCore, SyscallData, SyscallError,
};

use super::fs_obj::FSObjRef;

fn data_handler(fs: &FSFrontend, focus: Handle, data: String) -> Result<DaemonString, FSReturns> {
    log::debug!("FS: Client[{}] <- '$ {}'", focus.id, data);
    let r = FSCommand::try_from(data.clone())?;
    log::debug!("FS: Client[{}]: Command parsed successfully", focus.id);

    let ret: DaemonString = match r {
        FSCommand::List(path) => fs.list(path.to_string())?.join("?").into(),
        FSCommand::Get(s) => {
            let a = fs.get(s)?;
            let a = a.deref();
            let a: DaemonString = a.borrow().to_daemon_string()?;

            a
        }
        FSCommand::Set(s, obj) => {
            fs.set(s, obj)?;
            "Ok".into()
        }

        FSCommand::Stat(path) => fs.stat(path)?.to_daemon_string()?,
        FSCommand::Mkdir(path, name) => {
            fs.mkdir(path, name)?;
            "Ok".into()
        }
    };

    Ok(ret)
}

pub async fn fs_daemon_process(
    session: RustProcessCore,
    root: FSObjRef,
) -> Result<i64, SyscallError> {
    log::debug!("FS: Starting Daemon");

    let _s = session.ipc_create("system/file-system".to_string()).await?;
    log::debug!("FS: IPC Created!");

    let fs = FSFrontend::new(root.clone());

    loop {
        let data = session.get_syscall_data().await;
        match data {
            SyscallData::Connection { client, server: _ } => {
                log::debug!("FS: Client[{}] connected", client.id);
            }
            SyscallData::ReceivingData { focus, data } => {
                let ret = data_handler(&fs, focus.clone(), data);
                let ret = match ret {
                    Ok(ret) => ret.to_string(),
                    Err(e) => e.into(),
                };

                log::debug!("FS: Client[{}] -> {}", focus.id, ret);
                session.ipc_send(focus, ret).await?;
            }

            _ => {
                log::info!("Unknown SyscallData {data:?}");
                panic!();
            }
        }
    }
}
