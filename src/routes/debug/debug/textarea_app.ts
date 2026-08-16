import type { Process } from "$lib/session";


export async function textarea_main(p: Process, textarea: HTMLTextAreaElement) {
  await p.fs_mkdir("/", "srv");
  await p.fs_mkdir("/srv", "textarea");
  await p.fs_mkdir("/srv/textarea", "root");


  await p.fs_subscribe("/srv/textarea/root/push", (caller_pid: bigint, data: any) => {
    if (typeof data.String === "string") {
      textarea.value = data.String;
    } else {
      console.log("Invalid data received on /srv/textarea/root/push:", data);
    }
    return true;
  });

  await p.fs_subscribe("/srv/textarea/root/take", (caller_pid: bigint, data: any) => {
    if (typeof data.String !== "string") {
      console.log("Invalid pointer received on /srv/textarea/root/take:", data);
    } else {
      p.fs_publish(data.String, { String: textarea.value });
    }
    return true;
  });

  while (1) {
    await p.wait_for_event();
  }
}