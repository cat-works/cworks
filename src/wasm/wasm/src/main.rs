use std::{process::exit, sync::Arc};

use kernel::{RustProcess, Session, SyscallError};

extern crate kernel;
// extern crate python;

async fn client(session: Arc<Session<u32>>) -> Result<i64, SyscallError> {
    session.sleep(0.2).await;
    let c = session
        .ipc_connect("system/file-system".to_string())
        .await?;

    let mut n = 0;
    println!("Client: {}", c);
    session.ipc_send(c.clone(), "List".to_string()).await?;

    loop {
        let data = session.get_syscall_data().await;
        match data {
            kernel::SyscallData::ReceivingData { focus, data } if focus == c => {
                println!("C <- {}", data);
                if n == 0 {
                    session
                        // Cd?usr/mime/cafecode
                        .ipc_send(c.clone(), "Get?test".to_string())
                        .await?;
                    n = 1;
                } else if n == 1 {
                    session.ipc_send(c.clone(), "Get?text".to_string()).await?;
                    n = 2;
                } else if n == 2 {
                    session
                        .ipc_send(c.clone(), "Set?text?String?hi".to_string())
                        .await?;
                    n = 3;
                } else if n == 3 {
                    session.ipc_send(c.clone(), "Get?text".to_string()).await?;
                    n = 4;
                } else if n == 4 {
                    session
                        .ipc_send(c.clone(), "Get?handler".to_string())
                        .await?;
                    n = 5;
                } else if n == 5 {
                    exit(0);
                }
                // break;
            }

            _ => {
                println!("{data:?}");
                panic!();
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut k = kernel::Kernel::default();

    // k.register_process(Box::new(RustProcess::new(&server, 0)));
    k.register_process(Box::new(RustProcess::new(&client, 0)));

    k.start();
    Ok(())
}
