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
    const children_map: Map<String, any> = data.CompoundFSObj.children;

    if (typeof children_map.get("path")?.String !== "string") {
      console.warn("Invalid path received for exec-lua:", data);
      return
    }

    if (children_map.get("cwd") == undefined) {
      console.warn("Invalid params received for exec-lua:", data);
      return
    }
    if (children_map.get("cmd_line") == undefined) {
      console.warn("Invalid params received for exec-lua:", data);
      return
    }

    if (typeof children_map.get("stdout")?.String !== "string") {
      console.warn("Invalid stdout received for exec-lua:", data);
      return
    }

    if (typeof children_map.get("stdin")?.String !== "string") {
      console.warn("Invalid stdin received for exec-lua:", data);
      return
    }

    const path = children_map.get("path")?.String;
    const cwd = children_map.get("cwd")?.String;
    const cmd_line = children_map.get("cmd_line")?.String;
    const stdout = children_map.get("stdout")?.String;
    const stdin = children_map.get("stdin")?.String;
    const pid_reply_to = children_map.get("pid_reply_to")?.String;


    p.fs_get(path).then((data) => {
      if (typeof data.String !== "string") {
        console.warn("Invalid Lua code received for exec-lua:", data);
        return;
      }

      const process = new LuaProcess(path, data.String, {
        cwd,
        cmdline: cmd_line,
        stdout,
        stdin,
      });
      const pid = sess.add_process(process.kernel_callback.bind(process));

      if (pid_reply_to) {
        return p.fs_publish(pid_reply_to, { String: pid.toString() });
      }
    }).then(() => {
      console.debug("Lua process started for path:", path);
    });


    return true;
  });

  while (1) {
    await p.wait_for_event();
  }


  return;
}
