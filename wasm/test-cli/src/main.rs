use std::rc::Rc;

use kernel::{RustProcess, RustProcessCore, obj_tree::Object};

async fn server(mut session: RustProcessCore, _arg: u32) {
    const SOCK_PATH: &str = "/the-socket.ch";

    session
        .fs_set(
            "/sock-path".to_string(),
            Object::String(SOCK_PATH.to_string()).into(),
        )
        .await
        .expect("Failed to set sock-path");

    if let Err(e) = session
        .subscribe(
            SOCK_PATH.to_string(),
            Rc::new(Box::new(|data| {
                log::info!("Server received data: {data:?}");
                Ok(())
            })),
        )
        .await
    {
        log::error!("Server failed to subscribe: {e:?}");
        return;
    }

    session
        .wait_for_event()
        .await
        .expect("Failed to wait for event");
}

async fn client(session: RustProcessCore, _arg: u32) {
    session.sleep(0.2).await;
    let sock_path = session
        .fs_get("/sock-path".to_string())
        .await
        .expect("Failed to get sock-path");
    let sock_path = if let Object::String(ref s) = **sock_path.borrow() {
        s.clone()
    } else {
        log::error!("sock-path is not a string");
        return;
    };
    if let Err(e) = session.publish(sock_path, None).await {
        log::error!("Client failed to publish: {e:?}");
        return;
    }
    session.sleep(0.2).await;
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut k = kernel::Kernel::default();

    k.register_process(Box::new(RustProcess::new(&server, 0)));
    k.register_process(Box::new(RustProcess::new(&client, 0)));

    k.start();
}
