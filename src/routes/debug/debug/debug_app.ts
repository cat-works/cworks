import type { Handle, Process } from "$lib/session";

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
      console.log(`Received data: ${JSON.stringify(data)}`);
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

export async function debug_main(p: Process) {
  const stdio = new StdIO(p);
  await stdio.init("root");
  stdio.write(`[Debug APP]\n`);

  while (1) {
    const line = await stdio.readline();
    stdio.write(`Received: ${line}\n`);
  }

  return 0n;
}