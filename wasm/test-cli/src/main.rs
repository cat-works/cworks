use std::rc::Rc;

use kernel::{RustProcess, RustProcessCore, obj_tree::Object};

async fn server(mut session: RustProcessCore, _arg: u32) {
    if let Err(e) = session
        .subscribe(
            "/the-sock.ch".to_string(),
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
    if let Err(e) = session.publish("/the-sock.ch".to_string(), None).await {
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
