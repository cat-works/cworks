use std::process::exit;

use kernel::{obj_tree::Object, RustProcess, RustProcessCore, SyscallData, SyscallError};

mod generator;
mod js_process;
mod session;
extern crate kernel;

async fn server(session: RustProcessCore, _arg: u32) -> Result<i64, SyscallError> {
    session.subscribe("/the-sock.ch".to_string()).await?;

    loop {
        let data = session.get_syscall_data().await;
        if matches!(data, SyscallData::None) {
            continue;
        }

        log::info!("Server received data: {data:?}");
    }
}

async fn client(session: RustProcessCore, _arg: u32) -> Result<i64, SyscallError> {
    session.sleep(0.2).await;
    session.publish("/the-sock.ch".to_string(), None).await?;
    session.sleep(0.2).await;

    exit(0);
}
async fn fs_test(session: RustProcessCore, _arg: u32) -> Result<i64, SyscallError> {
    session
        .fs_set("/b".to_string(), Object::Int(1).into())
        .await
        .expect("Failed to set /b");

    // session.fs_stat("/".to_string()).await?;
    // session.fs_stat("/usr".to_string()).await?;
    // session.fs_stat("/usr/.".to_string()).await?;
    // session.fs_stat("/usr/..".to_string()).await?;
    session.fs_get("/usr".to_string()).await?;
    exit(0);
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut k = kernel::Kernel::default();

    k.register_process(Box::new(RustProcess::new(&server, 0)));
    k.register_process(Box::new(RustProcess::new(&client, 0)));
    k.register_process(Box::new(RustProcess::new(&fs_test, 0)));

    k.start();
    ()
}
