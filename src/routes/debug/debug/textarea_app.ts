import type { Process } from "$lib/session";


export async function textarea_main(p: Process, textarea: HTMLTextAreaElement) {
  await p.fs_mkdir("/", "run");
  await p.fs_mkdir("/run", "debug-app");


  await p.fs_subscribe("/run/debug-app/ta-push", (caller_pid: bigint, data: any) => {
    if (typeof data.String === "string") {
      textarea.value = data.String;
    } else {
      console.log("Invalid data received on /run/debug-app/ta-push:", data);
    }
    return true;
  });

  await p.fs_subscribe("/run/debug-app/ta-take", (caller_pid: bigint, data: any) => {
    if (typeof data.String !== "string") {
      console.log("Invalid pointer received on /run/debug-app/ta-take:", data);
    } else {
      p.fs_publish(data.String, { String: textarea.value });
    }
    return true;
  });

  // Stay running (like stdio_app) so requests queued by the subscribe handlers
  // (e.g. fs_publish in ta-take) are processed instead of being shadowed by a
  // stale WaitForEvent token in the result queue.
  while (1) {
    await p.pending();
  }
}