import type { Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";

import test_proc from "./test_proc.lua?raw";


export async function debug_main(p: Process, sess: Session) {
  const test_process = new LuaProcess(test_proc);
  sess.add_process(test_process.kernel_callback.bind(test_process));


  /*
  let pwd = '/';

  while (1) {
    if (command === "exec") {
      // Load string from args[0] into 'code'
      if (args.length === 0) {
        stdio.write("Usage: exec <filename>\n");
        continue;
      }
      const filename = `${pwd}${args[0]}`;
      const obj = await fs.get(filename);
      const code = obj.String;
      if (code === undefined) {
        stdio.write(`Error: Not a String file\n`);
        continue;
      }

      const test_process = new LuaProcess(code);
      sess.add_process(test_process.kernel_callback.bind(test_process));

    } else if (command === "ipc") {
      stdio.write(sess.get_ipc_names().join("\n") + "\n");
    }
  } */

  return 0n;
}