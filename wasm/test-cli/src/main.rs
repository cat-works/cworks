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
async fn fs_test(session: RustProcessCore, _arg: u32) {
    session
        .fs_set("/b".to_string(), Object::Int(1).into())
        .await
        .expect("Failed to set /b");

    session.sleep(1.0).await;

    match session.fs_get("/".to_string()).await {
        Err(e) => {
            log::error!("Failed to get /: {e:?}");
        }
        Ok(v) => match serde_json::to_string(&v) {
            Ok(json) => log::info!("Got /: {json}"),
            Err(e) => log::error!("Failed to serialize /: {e:?}"),
        },
    }
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
}
