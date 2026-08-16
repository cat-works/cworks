import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";

import test_proc from "./test_proc.lua?raw";


export async function debug_main(p: Process, sess: Session) {
  const test_process = new LuaProcess(test_proc);
  sess.add_process(test_process.kernel_callback.bind(test_process));
}