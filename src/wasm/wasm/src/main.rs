use std::sync::Arc;

use rust_process::{RustProcess, Session};

extern crate kernel;
// extern crate python;

async fn p(session: Arc<Session>) -> i64 {
    let a = session.ipc_create("aiueo".to_string()).await;
    let n = session.ipc_create("neptune".to_string()).await;

    println!("a: {}", a);
    println!("n: {}", n);

    0i64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();

    let p = RustProcess::new(&p);
    k.register_process(Box::new(p));

    k.start();
    Ok(())
}
