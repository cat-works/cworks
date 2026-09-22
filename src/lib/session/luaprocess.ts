import { LuaEnv } from "../../../wasm/cworks-wasm/pkg/cworks";
import stdio from "$lib/lua/stdio.lua?raw";
import bootstrap from "$lib/lua/bootstrap.lua?raw";
import json from "$lib/lua/usr/lib/json.lua?raw";

const env = new LuaEnv();
env.run(json);
env.run(stdio);
env.run(bootstrap);

export function createLuaProcess(name: string, code: string): bigint {
  return env.createProcess(name, code)
}
