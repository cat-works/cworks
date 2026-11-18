import type { Process } from "$lib/session";
import edit_lua from "$lib/lua/usr/bin/edit.lua?raw";
import wm_lua from "$lib/lua/usr/bin/wm.lua?raw";
import term_lua from "$lib/lua/usr/bin/term.lua?raw";
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

  {
    const ns = root.ensureCompound("run").ensureCompound("debug-app");
    const stdout = ns.ensureFunc("shell-out");
    const stdin = ns.ensureFunc("shell-in");
    const req = FSObj.newCompound();
    req.createString("name", "term.lua");
    req.createString("code", term_lua);
    req.createString("cmd_line", "sh");
    req.createString("cwd", "/");
    req.addChild("stdout", stdout);
    req.addChild("stdin", stdin);

    await execLua.publish(req);
  }

  {
    const ns = root.ensureCompound("run").ensureCompound("wm");
    const stdout = ns.ensureFunc("stdout");
    const stdin = ns.ensureFunc("stdin");

    const req = FSObj.newCompound();
    req.createString("name", "wm.lua");
    req.createString("code", wm_lua);
    req.createString("cmd_line", "wm");
    req.createString("cwd", "/");
    req.addChild("stdout", stdout);
    req.addChild("stdin", stdin);

    await execLua.publish(req);
  }
}
