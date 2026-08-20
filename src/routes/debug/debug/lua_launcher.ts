import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";


export async function lua_launcher(p: Process, sess: Session) {
  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "sys");

  await p.fs_subscribe("/run/sys/exec-lua", (caller_pid: bigint, data: any) => {
    if (!data.CompoundFSObj) {
      console.warn("Invalid data received for exec-lua:", data);
      return
    }
    const children = data.CompoundFSObj.children;

    if (typeof children.path?.String !== "string") {
      console.warn("Invalid path received for exec-lua:", children.path);
      return
    }

    if (children.params == undefined) {
      console.warn("Invalid params received for exec-lua:", children.params);
      return
    }

    const path = children.path.String;
    const params = children.params;

    const process = new LuaProcess(path, params);
    sess.add_process(process.kernel_callback.bind(process));

    return true;
  });

  while (1) {
    await p.wait_for_event();
  }


  return;
}