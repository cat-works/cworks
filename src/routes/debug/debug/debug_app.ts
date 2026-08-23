import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../../wasm/wasm/pkg/cworks";

import sh_lua from "$lib/lua/usr/bin/sh.lua?raw";

export async function debug_main(p: Process, sess: Session) {
  const scripts = import.meta.glob("$lib/lua/usr/**/*.lua", { eager: true, query: "?raw" });
  for (const [path, module] of Object.entries(scripts)) {
    const fs_path = path.replace(/^.*(\/usr\/.*)$/, "$1");
    const content = (module as any).default;
    await p.fs_set(fs_path, { String: content });
  }


  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "debug-app");


  const test_process = new LuaProcess("shell.lua", sh_lua, {
    cmdline: "sh",
    cwd: "/",
    stdout: "/run/debug-app/shell-out",
    stdin: "/run/debug-app/shell-in",
  });
  sess.add_process(test_process.kernel_callback.bind(test_process));
}
