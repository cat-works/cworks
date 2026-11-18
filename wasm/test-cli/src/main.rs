use kernel::{
    ProcessSession, RustProcess, SyscallError,
    obj_tree::{FSObjRef, Object},
};

use cworks_lua::new_lua_process;
use kernel::Kernel;
use mlua::chunk;
use mlua::prelude::*;

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

async fn eval_mock(mut session: ProcessSession, _arg: u32) {
    let root = session.fs_root().await.expect("fs_root failed");
    let srv = get_obj(&root, "srv", || FSObjRef::new_compound(&root))
        .expect("srv namespace creation failed");
    let eval_ns = get_obj(&srv, "eval", || FSObjRef::new_compound(&srv))
        .expect("eval namespace creation failed");
    let request_channel = get_obj(&eval_ns, "req", || FSObjRef::from(Object::Func(vec![])))
        .expect("req channel creation failed");

    request_channel
        .subscribe(Box::new(move |data: &FSObjRef| {
            let eval_ns = eval_ns.clone();

            let Object::String(ref string) = **data.borrow() else {
                log::warn!("eval_mock: invalid data type: {data:?}");
                return Box::pin(async {});
            };
            let string = string.clone();

            Box::pin(async move {
                get_obj(&eval_ns, "res", || FSObjRef::from(Object::Func(vec![])))
                    .expect("res channel creation failed")
                    .publish(Object::String(format!("eval reply: {string}")).into())
                    .await
                    .expect("publish failed");
            })
        }))
        .expect("subscribe failed");

    loop {
        session.wait_for_event().await.expect("wait failed");
    }
}

async fn eval_client(mut session: ProcessSession, _arg: u32) {
    let root = session.fs_root().await.expect("fs_root failed");
    let srv = get_obj(&root, "srv", || FSObjRef::new_compound(&root))
        .expect("srv namespace creation failed");
    let eval_ns = get_obj(&srv, "eval", || FSObjRef::new_compound(&srv))
        .expect("eval namespace creation failed");

    session.sleep(0.2).await;
    let request_ch = eval_ns.get_obj("req").expect("req channel not found");
    let response_ch = get_obj(&eval_ns, "res", || FSObjRef::from(Object::Func(vec![])))
        .expect("res channel creation failed");

    response_ch
        .subscribe(Box::new(|data: &FSObjRef| {
            let s = format!("{data:?}");
            Box::pin(async move {
                log::info!("eval_client: eval reply ====> {s}");
            })
        }))
        .expect("subscribe failed");

    session.sleep(0.2).await;

    let expr = Object::String("2+3*4".to_string()).into();
    request_ch.publish(expr).await.expect("publish failed");

    loop {
        session.wait_for_event().await.expect("wait failed");
    }
}

fn lua_test(kernel: &mut Kernel, lua: &Lua) {
    let lua_func = lua
        .load(chunk! {
            function(sess)
                local function ls_rec(obj, depth)
                    for k, v in pairs(obj:list()) do
                        if v == "." or v == ".." then
                            goto continue
                        end
                        local child = obj:get_obj(v)
                        print(string.rep("  ", depth) .. v .. " (" .. child:type() .. ")")
                        if child:type() == "CompoundFSObj" then
                            ls_rec(child, depth + 1)
                        end

                        ::continue::
                    end
                end

                local root = sess:fs_root()
                local req = root
                    :ensure_compound("srv")
                    :ensure_compound("eval")
                    :create_func("req")

                req:subscribe(function(data)
                    local type = data:type()
                    if type ~= "String" then
                        print("eval request: [" .. data:type() .. "]")
                        goto continue
                    end

                    print("eval request: " .. data:string())
                    local res_ch = root
                        :get_obj("srv")
                        :get_obj("eval")
                        :ensure_func("res")
                    local res_v = root.new_string("12");
                    print("<=============================================>")
                    res_ch:publish(res_v)


                    ::continue::
                end)

                ls_rec(root, 0)

                while true do
                    sess:wait_for_event()
                end
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
