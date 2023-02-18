use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::Future;

use crate::{
    fs::{FSObj, RefOrVal},
    rust_process::{RustProcess, Session},
    PollResult, Process, SyscallData, SyscallError,
};

struct FSDaemon {
    pub fs_root: Arc<FSObj>,
}

async fn fs_daemon(session: Arc<Session>, daemon: Arc<Mutex<FSObj>>) -> Result<i64, SyscallError> {
    let s = session.ipc_create("0system/fs".to_string()).await?;
    println!("Server: {}", s);
    let mut sc = None;

    loop {
        let data = session.get_syscall_data().await;
        match data {
            SyscallData::Connection { client, server } => {
                println!("Connection");
                println!("  Client: {}", client);
                println!("  Server: {}", server);

                if server == s {
                    sc = Option::Some(client);
                }
            }
            SyscallData::ReceivingData { focus, data } if Option::Some(focus.clone()) == sc => {
                println!("Received: {} [Client -> Server]", data);

                session
                    .ipc_send(focus.clone(), "DAT-KITAYO".to_string())
                    .await?;
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

impl FSDaemon {
    fn new() -> Box<dyn Process> {
        let root = Arc::new(Mutex::new(FSObj::Dist(RefOrVal::Val(HashMap::new()))));
        Box::new(RustProcess::new(&fs_daemon, root.clone()))
    }
}
