import { FileSystem } from "$lib/fs_wrapper";
import type { Handle, Process } from "$lib/session";
import { LuaProcess } from "$lib/session/luaprocess";
import type { Session } from "../../../wasm/pkg/wasm";
import { manuals } from "./man";

import test_proc from "./test_proc.lua?raw";

class StdIO {
  public ipc: Handle | null = null;
  public stdin_buffer: string = "";

  constructor(public proc: Process) { }

  async init(tag: string) {
    console.log(`Connecting to system/stdio/${tag}`);
    this.ipc = await this.proc.ipc_connect(`system/stdio/${tag}`);
  }

  async readline() {
    if (!this.ipc) {
      throw new Error("IPC not initialized");
    }

    while (!this.stdin_buffer.includes("\n")) {
      const data = await this.ipc.recv();
      this.stdin_buffer += data;
    }

    const index = this.stdin_buffer.indexOf("\n");
    const line = this.stdin_buffer.slice(0, index);
    this.stdin_buffer = this.stdin_buffer.slice(index + 1);
    return line;
  }

  async write(data: string) {
    if (!this.ipc) {
      throw new Error("IPC not initialized");
    }

    await this.ipc.send(data);
  }
}
class CodeEditor {
  public ipc: Handle | null = null;

  constructor(public proc: Process) { }

  async init(tag: string) {
    console.log(`Connecting to system/textarea/${tag}`);
    this.ipc = await this.proc.ipc_connect(`system/textarea/${tag}`);
  }

  async load(text: string) {
    if (!this.ipc) {
      throw new Error("IPC not initialized");
    }

    const payload = 'l' + text;
    await this.ipc.send(payload);
  }

  async acquire() {
    if (!this.ipc) {
      throw new Error("IPC not initialized");
    }

    const payload = 'a';
    await this.ipc.send(payload);

    const data = await this.ipc.recv();
    if (data.startsWith('l')) {
      return data.slice(1); // Remove the 'l' prefix
    } else {
      throw new Error("Invalid data received from IPC");
    }
  }
}

export async function debug_main(p: Process, sess: Session) {
  const stdio = new StdIO(p);
  await stdio.init("root");

  const editor = new CodeEditor(p);
  await editor.init("root");

  const fs = new FileSystem(p);
  await fs.wait_for_ready();

  {
    await fs.set_raw("/test.lua", "String?" + test_proc);

    await fs.mkdir("/usr", "lib");
  }

  stdio.write(`\x1b[1;32mCat OS Shell\x1b[m\n`);
  stdio.write(`Type 'man commands' to see available commands.\n\n`);


  let pwd = '/';

  while (1) {
    stdio.write(`\x1b[1;32m${pwd}\x1b[m\n`);
    stdio.write(`\x1b[2m$\x1b[m `);
    const line = await stdio.readline();

    const [command, ...args] = line.split(" ");

    try {
      if (command === "ls") {
        const entries = await fs.list(pwd);
        for (const entry of entries) {
          try {
            let s = await fs.stat(`${pwd}${entry}`);
            stdio.write(`${s} ${entry}\n`);
          } catch {
            stdio.write(`? ${entry}\n`);

          }
        }
      } else if (command === "cd") {
        if (args[0]) {
          pwd += args[0];
          if (!pwd.endsWith('/')) {
            pwd += '/';
          }
        } else {
          pwd = '/';
        }
      } else if (command === "mkdir") {
        await fs.mkdir(pwd, args[0]);
      } else if (command === "stat") {
        try {
          const stat = await fs.stat(`${pwd}${args[0]}`);
          stdio.write(`${stat}\n`);
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "get") {
        try {
          const stat = await fs.get(`${pwd}${args[0]}`);
          stdio.write(`${stat}\n`);
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "man") {
        if (args[0] && manuals[args[0]]) {
          stdio.write(manuals[args[0]] + "\n");
        }
        else {
          stdio.write("Available manuals:\n");
          for (const key in manuals) {
            stdio.write(`- ${key}\n`);
          }
        }
      } else if (command === "set") {
        try {
          await fs.set_raw(`${pwd}${args[0]}`, args.slice(1).join(" "));
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "cat") {
        try {
          const [kind, content] = await fs.get(`${pwd}${args[0]}`);
          if (kind !== "String") {
            stdio.write(`Error: Not a String file\n`);
          }

          stdio.write(content + "\n");
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "load") {
        try {
          const [kind, content] = await fs.get(`${pwd}${args[0]}`);
          if (kind !== "String") {
            stdio.write(`Error: Not a String file\n`);
          }

          await editor.load(content);
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "save") {
        try {
          const content = await editor.acquire();
          if (!content) {
            stdio.write(`Error: No content to save\n`);
            continue;
          }

          await fs.set_raw(`${pwd}${args[0]}`, "String?" + content);
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "clear") {
        stdio.write(`\x1b[2J\x1b[H`);
      } else if (command === "exec") {
        // Load string from args[0] into 'code'
        if (args.length === 0) {
          stdio.write("Usage: exec <filename>\n");
          continue;
        }
        const filename = `${pwd}${args[0]}`;
        try {
          const [kind, code] = await fs.get(filename);
          if (kind !== "String") {
            stdio.write(`Error: Not a String file\n`);
            continue;
          }

          const test_process = new LuaProcess(code);
          sess.add_process(test_process.kernel_callback.bind(test_process));
        } catch (e) {
          stdio.write(`Error: ${e}\n`);
        }
      } else if (command === "ipc") {
        stdio.write(sess.get_ipc_names().join("\n") + "\n");
      }
    } catch (e) {
      stdio.write(`Error: ${e}\n`);
    }
  }

  return 0n;
}