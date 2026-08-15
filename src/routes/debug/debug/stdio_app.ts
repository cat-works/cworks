import type { Process } from "$lib/session";


export async function stdio_main(p: Process, terminal: { stdin: () => Promise<string>, write: (data: string) => void }) {
  await p.fs_mkdir("/", "srv");
  await p.fs_mkdir("/srv", "stdio");
  await p.fs_mkdir("/srv/stdio", "root");

  await p.fs_subscribe("/srv/stdio/root/out", (caller_pid: bigint, data: any) => {
    if (typeof data.String === "string") {
      terminal.write(data.String);
    } else {
      console.error("Invalid data received on /srv/stdio/root/out:", data);
    }
    return true;
  });

  while (1) {
    const key = await terminal.stdin();
    terminal.write(key);

    p.fs_publish("/srv/stdio/root/in", { String: key });
  }


  return 0n;
}