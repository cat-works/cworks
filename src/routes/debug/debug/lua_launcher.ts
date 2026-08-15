import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";


export async function lua_launcher(p: Process, sess: Session) {
  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "sys");

  await p.fs_subscribe("/run/sys/exec-lua", (caller_pid: bigint, data: any) => {
    if (typeof data.String === "string") {
      const process = new LuaProcess(data.String);
      sess.add_process(process.kernel_callback.bind(process));
    }
    return true;
  });

  while (1) {
    await p.pending();
  }


  return 0n;
}