use std::sync::Arc;

use kernel::{rust_process::*, SyscallError};

extern crate kernel;
// extern crate python;

async fn server(session: Arc<Session>, _: u32) -> Result<i64, SyscallError> {
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

async fn client(session: Arc<Session>, _: u32) -> Result<i64, SyscallError> {
    session.sleep(0.5).await;
    let c = session.ipc_connect("aiueo".to_string()).await?;
    session
        .ipc_send(c.clone(), "Helloworld".to_string())
        .await?;

    println!("Client: {}", c);

    loop {
        let data = session.get_syscall_data().await;
        match data {
            kernel::SyscallData::ReceivingData { focus, data } if focus == c => {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();

    k.register_process(Box::new(RustProcess::new(&server, 0)));
    k.register_process(Box::new(RustProcess::new(&client, 0)));

    k.start();
    Ok(())
}
