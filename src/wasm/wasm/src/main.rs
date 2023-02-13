use std::sync::Arc;

use kernel::SyscallError;
use rust_process::{RustProcess, Session};

extern crate kernel;
// extern crate python;

async fn p(session: Arc<Session>) -> Result<i64, SyscallError> {
    let s = session.ipc_create("aiueo".to_string()).await?;
    println!("Server: {}", s);
    let c = session.ipc_connect("aiueo".to_string()).await?;
    println!("Client: {}", c);

    while let data = session.get_syscall_data() {
        match data {
            kernel::SyscallData::ReceivingData { focus, data } if focus == s => {
                println!("Received: {} [Server]", data);
            }

            kernel::SyscallData::ReceivingData { focus, data } if focus == c => {
                println!("Received: {} [Client]", data);
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

    let p = RustProcess::new(&p);
    k.register_process(Box::new(p));

    k.start();
    Ok(())
}
