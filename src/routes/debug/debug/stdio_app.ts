import type { Handle, Process } from "$lib/session";


export async function stdio_main(p: Process, terminal: { stdin: () => Promise<string>, write: (data: string) => void }) {
  const server = await p.ipc_create("system/stdio/root");

  let clients_active: Handle[] = [];
  let primary: Handle | undefined = undefined;

  server.on("connection", (h: Handle) => {
    clients_active.push(h);
    if (primary === undefined) {
      primary = h;
    }

    h.on("message", (data: string) => {
      // inactive code: \x1b(0
      // active code: \x1b(1
      if (data === "\x1b(0") {
        clients_active = clients_active.filter((x) => x.handle.id !== h.handle.id);
      } else if (data === "\x1b(1") {
        clients_active = clients_active.filter((x) => x.handle.id !== h.handle.id);
        clients_active.push(h);
      }

      terminal.write(data);
      return true;
    });

    return true;
  });


  async function stdin_process() {
    while (1) {
      const key = await terminal.stdin();
      terminal.write(key);

      if (clients_active.length > 0) {
        for (const client of clients_active) {
          client.send(key);
        }
      } else if (primary) {
        primary.send(key);
      } else {
        terminal.write(`\x1b[2m${key}\x1b[0m\n`);
      }
    }
  }

  await Promise.all([stdin_process()]);


  return 0n;
}