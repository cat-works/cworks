import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../../wasm/lua/pkg/cworks";


export async function lua_launcher(p: Process, sess: Session) {
  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "sys");

  await p.fs_subscribe("/run/sys/exec-lua", (caller_pid: bigint, data: any) => {
    if (!data.CompoundFSObj) {
      console.warn("Invalid data received for exec-lua:", data);
      return
    }
    const children: Record<string, any> = data.CompoundFSObj.children;

    if (typeof children["path"]?.String !== "string") {
      console.warn("Invalid path received for exec-lua:", data);
      return
    }

    if (children["cwd"] == undefined) {
      console.warn("Invalid params received for exec-lua:", data);
      return
    }
    if (children["cmd_line"] == undefined) {
      console.warn("Invalid params received for exec-lua:", data);
      return
    }

    if (typeof children["stdout"]?.String !== "string") {
      console.warn("Invalid stdout received for exec-lua:", data);
      return
    }

    if (typeof children["stdin"]?.String !== "string") {
      console.warn("Invalid stdin received for exec-lua:", data);
      return
    }

    const path = children["path"]?.String;
    const cwd = children["cwd"]?.String;
    const cmd_line = children["cmd_line"]?.String;
    const stdout = children["stdout"]?.String;
    const stdin = children["stdin"]?.String;
    const pid_reply_to = children["pid_reply_to"]?.String;


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
