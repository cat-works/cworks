import { createNativeLuaProcess, LuaEnv } from "../../../wasm/wasm/pkg/cworks";
import stdio from "$lib/lua/stdio.lua?raw";
import bootstrap from "$lib/lua/bootstrap.lua?raw";
import json from "$lib/lua/usr/lib/json.lua?raw";

const env = new LuaEnv();
env.run(json);
env.run(stdio);
env.run(bootstrap);

export interface LuaProcessOpts {
  cwd: string;
  cmdline: string;
  stdout: string;
  stdin: string;
  extra_args?: any;
}

interface LuaProcessEnv {
  cmdline: string;
  cwd: string;
  stdout: string;
  stdin: string;
  extra_args: any;
}

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

function luaKey(k: string): string {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(k) ? k : longBracket(k);
}

// Serialize a JSON-like value to a Lua table literal.
function toLuaLiteral(v: any): string {
  if (v === null || v === undefined) return "nil";
  switch (typeof v) {
    case "string":
      return longBracket(v);
    case "number":
      return Number.isFinite(v) ? String(v) : "nil";
    case "boolean":
      return v ? "true" : "false";
    case "object": {
      if (Array.isArray(v)) {
        return "{" + v.map(toLuaLiteral).join(", ") + "}";
      }
      const parts: string[] = [];
      for (const [k, val] of Object.entries(v)) {
        parts.push(luaKey(k) + " = " + toLuaLiteral(val));
      }
      return "{ " + parts.join(", ") + " }";
    }
    default:
      throw new Error(
        "LuaProcess: cannot serialize value to Lua literal: " + typeof v,
      );
  }
}

function buildThinChunk(env: LuaProcessEnv, code: string, name: string): string {
  return (
    "local env = " +
    toLuaLiteral(env) +
    "\n" +
    'package.loaded["__bootstrap"].run(env, ' +
    longBracket(code) +
    ", " +
    longBracket(name) +
    ")\n"
  );
}

export class LuaProcess {
  public pid: bigint;

  constructor(name: string, code: string, opts: LuaProcessOpts) {
    const { cmdline, cwd, stdout, stdin } = opts;
    if (
      typeof cmdline !== "string" ||
      typeof cwd !== "string" ||
      typeof stdout !== "string" ||
      typeof stdin !== "string"
    ) {
      throw new Error(
        "LuaProcess: opts.cmdline, opts.cwd, opts.stdout, opts.stdin are required",
      );
    }
    if (stdin === "") {
      throw new Error("LuaProcess: opts.stdin must be a channel path (empty string not allowed)");
    }

    const procEnv: LuaProcessEnv = {
      cmdline,
      cwd,
      stdout,
      stdin,
      extra_args: opts.extra_args ?? {},
    };

    const chunk = buildThinChunk(procEnv, code, name);

    const nativeProcess = createNativeLuaProcess(env, name, chunk);
    this.pid = nativeProcess;
  }
}
