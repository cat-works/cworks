//! Native playground for testing cworks kernel features.

use std::rc::Rc;

use kernel::{
    ProcessClientExt, RustProcess, RustProcessCore,
    obj_tree::{FSObjRef, Object},
};
use plugin::WasmPluginProcess;

mod plugin;

/// eval プラグインの E2E: 式を publish し、応答を subscribe して検証する。
async fn eval_client(mut session: RustProcessCore, _arg: u32) {
    // プラグイン側の mkdir/subscribe 完了を待つ
    session.sleep(0.3).await;

    let handler: Rc<Box<dyn Fn(Option<FSObjRef>) -> Result<(), kernel::SyscallError>>> =
        Rc::new(Box::new(|data| {
            log::info!("eval reply: {data:?}");
            Ok(())
        }));
    session
        .subscribe("/srv/eval/res".to_string(), handler)
        .await
        .expect("subscribe failed");

    // プラグインが subscribe を完了するのを待つ
    session.sleep(0.2).await;

    let expr = Object::String("2+3*4".to_string()).into();
    session
        .publish("/srv/eval/req".to_string(), Some(expr))
        .await
        .expect("publish failed");

    // 応答の到達を待つ
    for _ in 0..10 {
        session.wait_for_event().await.expect("wait failed");
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let wasm = include_bytes!("../../target/wasm32-unknown-unknown/debug/plugin_eval.wasm");

    let mut k = kernel::Kernel::default();
    k.set_process_debug(true);

    match WasmPluginProcess::load(wasm) {
        Ok(plugin) => {
            log::info!("plugin loaded");
            k.register_process(Box::new(plugin));
        }
        Err(e) => log::error!("plugin load failed: {e}"),
    }

    k.register_process(Box::new(RustProcess::new(&eval_client, 0)));

    k.start();
}
