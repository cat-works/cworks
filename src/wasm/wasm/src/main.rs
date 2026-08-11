use std::process::exit;

use kernel::{obj_tree::IntrinsicFSObj, RustProcess, RustProcessCore, SyscallError};

mod generator;
mod js_process;
mod session;
extern crate kernel;

async fn client(session: RustProcessCore, _arg: u32) -> Result<i64, SyscallError> {
    session.sleep(0.2).await;

    session
        .fs_set("/b".to_string(), IntrinsicFSObj::Int(1).into())
        .await
        .expect("Failed to set /b");

    session.fs_stat("/".to_string()).await?;
    session.fs_stat("/workspace".to_string()).await?;
    session.fs_stat("/mnt".to_string()).await?;
    session.fs_stat("/usr".to_string()).await?;
    session.fs_list("/usr".to_string()).await?;
    session.fs_stat("/usr/.".to_string()).await?;
    session.fs_stat("/usr/..".to_string()).await?;
    exit(0);
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
