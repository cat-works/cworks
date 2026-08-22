import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";

import test_proc from "./test_proc.lua?raw";
import json_lua from "$lib/lua/json.lua?raw";
import ls_lua from "$lib/lua/ls.lua?raw";


export async function debug_main(p: Process, sess: Session) {
  await p.fs_set("/usr/lib/json.lua", { String: json_lua });
  await p.fs_set("/usr/bin/shell.lua", { String: test_proc });
  await p.fs_set("/usr/bin/ls.lua", { String: ls_lua });

  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "debug-app");

  const test_process = new LuaProcess("shell.lua", test_proc, {
    cmdline: "sh",
    stdout: "/run/debug-app/shell-out",
    stdin: "/run/debug-app/shell-in",
  });
  sess.add_process(test_process.kernel_callback.bind(test_process));
}
