import { FileSystem } from "$lib/fs_wrapper";
import type { Handle, Process } from "$lib/session";
import type { Session } from "../../../wasm/pkg/wasm";

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

export async function debug_main(p: Process, sess: Session) {
  const stdio = new StdIO(p);
  await stdio.init("root");

  const fs = new FileSystem(p);
  await fs.wait_for_ready();
  stdio.write(`[Debug APP]\n`);

  let pwd = '/';

  while (1) {
    stdio.write(`\x1b[32m${pwd}\x1b[m\n`);
    stdio.write(`\x1b[1m$\x1b[m `);
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
        pwd += args[0] || '/';
        if (!pwd.endsWith('/')) {
          pwd += '/';
        }
      } else if (command === "mkdir") {
        await fs.mkdir(pwd, args[0]);
      } else if (command === "root") {
        pwd = '/';
      } else if (command === "ipc") {
        stdio.write(sess.get_ipc_names().join("\n") + "\n");
      }
    } catch (e) {
      stdio.write(`Error: ${e}\n`);
    }
  }

  return 0n;
}