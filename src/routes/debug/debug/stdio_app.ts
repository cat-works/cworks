import type { Process } from "$lib/session";


export async function stdio_main(p: Process, terminal: { stdin: () => Promise<string>, write: (data: string) => void }) {
  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "debug-app");

  await p.fs_subscribe("/run/debug-app/shell-out", (caller_pid: bigint, data: any) => {
    if (typeof data.String === "string") {
      terminal.write(data.String);
    } else {
      console.error("Invalid data received on /run/debug-app/shell-out:", data);
    }
    return true;
  });

  while (1) {
    const key = await terminal.stdin();
    terminal.write(key);

    p.fs_publish("/run/debug-app/shell-in", { String: key });
  }
}