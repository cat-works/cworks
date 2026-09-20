//! Native playground for testing cworks kernel features.

use std::rc::Rc;

use cworks_lua::new_lua_process;
use kernel::{
    Kernel, ProcessClientExt, RustProcess, RustProcessCore, SyscallError,
    obj_tree::{FSObjRef, Object},
};
use mlua::prelude::*;

use mlua::chunk;

// mod plugin;
// use plugin::WasmPluginProcess;

fn get_obj(
    obj: &FSObjRef,
    name: &str,
    default: impl Fn() -> FSObjRef,
) -> Result<FSObjRef, SyscallError> {
    obj.get_obj(name).or_else(|_| {
        let child = default();
        obj.add_child(name, &child)?;
        Ok(child)
    })
}

async fn eval_mock(mut session: RustProcessCore, _arg: u32) {
    let root = session.fs_root().await.expect("fs_root failed");
    let srv = get_obj(&root, "srv", || {
        FSObjRef::from(Object::CompoundFSObj {
            parent: Some(root.clone()),
            children: std::collections::HashMap::new(),
        })
    })
    .expect("srv namespace creation failed");
    let eval_ns = get_obj(&srv, "eval", || {
        FSObjRef::from(Object::CompoundFSObj {
            parent: Some(srv.clone()),
            children: std::collections::HashMap::new(),
        })
    })
    .expect("eval namespace creation failed");
    let request_channel = get_obj(&eval_ns, "req", || {
        FSObjRef::from(Object::Func { callee_pid: vec![] })
    })
    .expect("req channel creation failed");

    session
        .subscribe(
            request_channel,
            Rc::new(Box::new(move |data| {
                if let Some(obj) = data
                    && let Object::String(s) = &**obj.borrow()
                {
                    log::info!("eval request: {s}");
                }
                Ok(())
            })),
        )
        .await
        .expect("subscribe failed");

    loop {
        session.wait_for_event().await.expect("wait failed");
    }
}

/// eval プラグインの E2E: 式を publish し、応答を subscribe して検証する。
async fn eval_client(mut session: RustProcessCore, _arg: u32) {
    let root = session.fs_root().await.expect("fs_root failed");
    let srv = get_obj(&root, "srv", || {
        FSObjRef::from(Object::CompoundFSObj {
            parent: Some(root.clone()),
            children: std::collections::HashMap::new(),
        })
    })
    .expect("srv namespace creation failed");
    let eval_ns = get_obj(&srv, "eval", || {
        FSObjRef::from(Object::CompoundFSObj {
            parent: Some(srv.clone()),
            children: std::collections::HashMap::new(),
        })
    })
    .expect("eval namespace creation failed");

    session.sleep(0.2).await;
    let request_ch = eval_ns.get_obj("req").expect("req channel not found");
    let response_ch = get_obj(&eval_ns, "res", || {
        FSObjRef::from(Object::Func { callee_pid: vec![] })
    })
    .expect("res channel creation failed");

    session
        .subscribe(
            response_ch,
            Rc::new(Box::new(|data| {
                log::info!("eval reply: {data:?}");
                Ok(())
            })),
        )
        .await
        .expect("subscribe failed");

    // プラグインが subscribe を完了するのを待つ
    session.sleep(0.2).await;

    let expr = Object::String("2+3*4".to_string()).into();
    session
        .publish(request_ch, Some(expr))
        .await
        .expect("publish failed");

    // 応答の到達を待つ
    for _ in 0..10 {
        session.wait_for_event().await.expect("wait failed");
    }
}

fn lua_test(kernel: &mut Kernel, lua: &Lua) {
    let lua_func = lua
        .load(chunk! {
            function(sess)
                local function ls_rec(obj, depth)
                    for k, v in pairs(obj:list()) do
                        if v ~= "." and v ~= ".." then
                            local child = obj:get_obj(v)
                            print(string.rep("  ", depth) .. v .. " (" .. child:type() .. ")")
                            if child:type() == "CompoundFSObj" then
                                ls_rec(child, depth + 1)
                            end
                        end
                    end
                end

                local root = sess:fs_root()
                local req = root
                    :ensure_compound("srv")
                    :ensure_compound("eval")
                    :create_func("req")

                sess:subscribe(req, function(data)
                    local type = data:type()
                    if type == "String" then
                        print("eval request: " .. data:string())
                        local res_ch = root
                            :get_obj("srv")
                            :get_obj("eval")
                            :ensure_func("res")
                        local res_v = root.new_string("12");
                        sess:publish(res_ch, resv)
                    else
                        print("eval request: [" .. data:type() .. "]")
                    end
                end)

                ls_rec(root, 0)

                sess:wait_for_event()
            end
        })
        .eval::<mlua::Function>()
        .expect("failed to eval lua chunk");

    let lua_proc = new_lua_process(lua_func).expect("failed to create lua process");
    kernel.register_process(Box::new(lua_proc));
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut k = kernel::Kernel::default();
    k.set_process_debug(true);

    let lua = mlua::Lua::new();
    lua_test(&mut k, &lua);

    // k.register_process(Box::new(RustProcess::new(&eval_mock, 0)));
    /* let wasm = include_bytes!("../../target/wasm32-unknown-unknown/debug/plugin_eval.wasm");
    match WasmPluginProcess::load(wasm) {
        Ok(plugin) => {
            log::info!("plugin loaded");
            k.register_process(Box::new(plugin));
        }
        Err(e) => log::error!("plugin load failed: {e}"),
    } */

    k.register_process(Box::new(RustProcess::new(&eval_client, 0)));

    k.start();
}
