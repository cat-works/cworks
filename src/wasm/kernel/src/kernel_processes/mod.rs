use std::sync::{Arc, Mutex};

use crate::{fs::FSObj, rust_process::Session, SyscallData, SyscallError};

enum FSReturns {
    InvalidCommandFormat,
}

impl From<FSReturns> for String {
    fn from(value: FSReturns) -> Self {
        match value {
            FSReturns::InvalidCommandFormat => "InvalidCommandFormat".to_string(),
        }
    }
}

enum FSCommand {
    List,
}
impl TryFrom<String> for FSCommand {
    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "l" => Ok(FSCommand::List),
            _ => panic!(),
        }
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

// fn fs_daemon() -> Box<dyn Process> {
//     let fs_obj = ;
//
//     Box::new(RustProcess::new(&fs_daemon_process, fs_obj))
// }
