import type { Process } from "$lib/session";
import sh_lua from "$lib/lua/usr/bin/sh.lua?raw";
import wm_lua from "$lib/lua/usr/bin/wm.lua?raw";
import { FSObj } from "../../../../wasm/cworks-wasm/pkg/cworks";

export async function debug_main(p: Process) {
  const root = await p.fsRoot();

  const scripts = import.meta.glob("$lib/lua/usr/**/*.lua", {
    eager: true,
    query: "?raw",
  });
  for (const [path, module] of Object.entries(scripts)) {
    const fs_path = path.replace(/^.*(\/usr\/.*)$/, "$1");
    const content = (module as any).default;
    const parts = fs_path.split("/").filter((part: string) => part);
    const name = parts.pop() as string;
    let dir = root;
    for (const part of parts) {
      dir = dir.ensureCompound(part);
    }
    dir.createString(name, content);
  }

  const execLua = root.ensureCompound("run").ensureCompound("sys").ensureFunc("exec-lua");
  const debugApp = root.ensureCompound("run").ensureCompound("debug-app");
  const shellOut = debugApp.ensureFunc("shell-out");
  const shellIn = debugApp.ensureFunc("shell-in");

  const request = FSObj.newCompound();
  request.createString("name", "shell.lua");
  request.createString("code", sh_lua);
  request.createString("cmd_line", "sh");
  request.createString("cwd", "/");
  request.addChild("stdout", shellOut);
  request.addChild("stdin", shellIn);

  await p.publish(execLua, request);

  const wmNs = root.ensureCompound("run").ensureCompound("wm");
  const wmStdout = wmNs.ensureFunc("stdout");
  const wmStdin = wmNs.ensureFunc("stdin");

  const wmRequest = FSObj.newCompound();
  wmRequest.createString("name", "wm.lua");
  wmRequest.createString("code", wm_lua);
  wmRequest.createString("cmd_line", "wm");
  wmRequest.createString("cwd", "/");
  wmRequest.addChild("stdout", wmStdout);
  wmRequest.addChild("stdin", wmStdin);

  await p.publish(execLua, wmRequest);
}
