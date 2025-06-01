import type { Handle, Process } from "$lib/session";


export async function textarea_main(p: Process, textarea: HTMLTextAreaElement) {
  const server = await p.ipc_create("system/textarea/root");

  let client: Handle = await new Promise((resolve) => {
    server.on("connection", (h: Handle) => {
      resolve(h);
      return true;
    });
  });

  while (1) {
    const data = await client.recv();
    const [op, ...payload] = data.split("");

    switch (op) {
      case 'l': // load
        textarea.value = payload.join("");
        break;
      case 'a': // acquire
        client.send(`l${textarea.value}`);
        break;
    }
  }

  return 0n;
}