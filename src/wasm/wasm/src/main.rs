use std::process::exit;

use kernel::{RustProcess, RustProcessCore, SyscallError};

mod generator;
mod js_process;
mod session;
extern crate kernel;

async fn client(session: RustProcessCore, _arg: u32) -> Result<i64, SyscallError> {
    session.sleep(0.2).await;
    let c = session
        .ipc_connect("system/file-system".to_string())
        .await?;

    let mut n = 0;
    println!("Client: {}", c);
    session
        .ipc_send(c.clone(), "Set?/b?Integer?1".to_string())
        .await?;

    loop {
        let data = session.get_syscall_data().await;
        match data {
            kernel::SyscallData::ReceivingData { focus, data } if focus == c => {
                println!("C <- {}", data);
                if n == 0 {
                    session.ipc_send(c.clone(), "Stat?/.".to_string()).await?;
                    n = 7;
                } else if n == 1 {
                    session
                        .ipc_send(c.clone(), "Stat?/workspace".to_string())
                        .await?;
                    n = 2;
                } else if n == 2 {
                    session.ipc_send(c.clone(), "Stat?/mnt".to_string()).await?;
                    n = 3;
                } else if n == 3 {
                    session.ipc_send(c.clone(), "Stat?/usr".to_string()).await?;
                    n = 4;
                } else if n == 4 {
                    session
                        .ipc_send(c.clone(), "List?/usr/".to_string())
                        .await?;
                    n = 5;
                } else if n == 5 {
                    session
                        .ipc_send(c.clone(), "Stat?/usr/.".to_string())
                        .await?;
                    n = 6;
                } else if n == 6 {
                    session
                        .ipc_send(c.clone(), "Stat?/usr/..".to_string())
                        .await?;
                    n = 7;
                } else if n == 7 {
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
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut k = kernel::Kernel::default();

    // k.register_process(Box::new(RustProcess::new(&server, 0)));
    k.register_process(Box::new(RustProcess::new(&client, 0)));

    k.start();
    Ok(())
}
