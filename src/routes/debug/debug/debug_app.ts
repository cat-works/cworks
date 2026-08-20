import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";

import test_proc from "./test_proc.lua?raw";
import cworks_lua from "$lib/lua/cworks.lua?raw";
import json_lua from "$lib/lua/json.lua?raw";


export async function debug_main(p: Process, sess: Session) {
  await p.fs_set("/usr/lib/cworks.lua", { String: cworks_lua });
  await p.fs_set("/usr/lib/json.lua", { String: json_lua });
  await p.fs_set("/usr/bin/shell.lua", { String: test_proc });

  const test_process = new LuaProcess("shell.lua", test_proc, "sh");
  sess.add_process(test_process.kernel_callback.bind(test_process));
}