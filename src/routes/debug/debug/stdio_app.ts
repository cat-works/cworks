import type { Handle, Process } from "$lib/session";


export async function stdio_main(p: Process, terminal: { stdin: () => Promise<string>, write: (data: string) => void }) {
  const server = await p.ipc_create("system/stdio/root");

  let client: Handle = await new Promise((resolve) => {
    server.on("connection", (h: Handle) => {
      resolve(h);
      return true;
    });
  });

  async function stdout_process() {
    while (1) {
      const data = await client.recv();
      terminal.write(data);
    }
  }

  async function stdin_process() {
    while (1) {
      const key = await terminal.stdin();
      await client.send(key);
    }
  }

  await Promise.all([stdout_process(), stdin_process()]);


  return 0n;
}