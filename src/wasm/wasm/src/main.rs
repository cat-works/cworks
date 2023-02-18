use std::{process::exit, sync::Arc};

use kernel::{rust_process::*, SyscallError};

extern crate kernel;
// extern crate python;

async fn server(session: Arc<Session<u32>>) -> Result<i64, SyscallError> {
    let s = session.ipc_create("aiueo".to_string()).await?;
    println!("Server: {}", s);
    let mut sc = None;

    loop {
        let data = session.get_syscall_data().await;
        match data {
            kernel::SyscallData::Connection { client, server } => {
                println!("Connection");
                println!("  Client: {}", client);
                println!("  Server: {}", server);

                if server == s {
                    sc = Option::Some(client);
                }
            }
            kernel::SyscallData::ReceivingData { focus, data }
                if Option::Some(focus.clone()) == sc =>
            {
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
                println!("<- {}", data);
                if n == 0 {
                    session.ipc_send(c.clone(), "Cd?usr".to_string()).await?;
                    n = 1;
                } else if n == 1 {
                    session.ipc_send(c.clone(), "List".to_string()).await?;
                    n = 2;
                } else if n == 2 {
                    exit(0);
                    break;
                }
                // break;
            }

            _ => {
                println!("{data:?}");
                panic!();
            }
        }
    }

    Ok(0i64)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();

    // k.register_process(Box::new(RustProcess::new(&server, 0)));
    k.register_process(Box::new(RustProcess::new(&client, 0)));

    k.start();
    Ok(())
}
