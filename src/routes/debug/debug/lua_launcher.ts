import type { Process } from "$lib/session";
import { createLuaProcess } from "$lib/session/luaprocess";
import { FSObj } from "../../../../wasm/cworks-wasm/pkg/cworks";

// Long-bracket string literal safe for the given content: `[==[ ... ]==]`
function longBracket(s: string): string {
  const re = /\]\=+\]/g;
  let maxEq = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(s)) !== null) {
    const eq = m[0].length - 2;
    if (eq > maxEq) maxEq = eq;
  }
  const level = maxEq + 1;
  return "[" + "=".repeat(level) + "[" + s + "]" + "=".repeat(level) + "]";
}

function buildThinChunk(code: string, name: string): string {
  return (
    "local session = ...\n" +
    'package.loaded["__bootstrap"].run(session, ' +
    longBracket(code) +
    ", " +
    longBracket(name) +
    ")\n"
  );
}

export async function lua_launcher(p: Process) {
  const root = await p.fsRoot();
  const execLua = root.ensureCompound("run").ensureCompound("sys").ensureFunc("exec-lua");
  const procNs = root.ensureCompound("run").ensureCompound("proc");

  await p.subscribe(execLua, (arg) => {
    if (arg.type() !== "CompoundFSObj") {
      console.warn("Invalid data received for exec-lua:", arg);
      return;
    }

    const name = arg.getObj("name").string();
    const code = arg.getObj("code").string();
    const pidReplyTo = arg.hasChild("pid_reply_to") ? arg.getObj("pid_reply_to") : null;

    queueMicrotask(() => {
      const pid = createLuaProcess(name, buildThinChunk(code, name));
      procNs.addChild(pid.toString(), arg);
      if (pidReplyTo) {
        p.publish(pidReplyTo, FSObj.newInt(Number(pid)));
      }
    });
  });

  while (1) {
    await p.waitForEvent();
  }
}
