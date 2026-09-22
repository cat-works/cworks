import type { Process } from "$lib/session";
import { FSObj } from "../../../../wasm/cworks-wasm/pkg/cworks";

export async function textarea_main(
  p: Process,
  textarea: HTMLTextAreaElement,
) {
  const root = await p.fsRoot();
  const debugApp = root
    .ensureCompound("run")
    .ensureCompound("debug-app");

  const taPush = debugApp.ensureFunc("ta-push");
  const taTake = debugApp.ensureFunc("ta-take");

  await p.subscribe(taPush, (arg) => {
    if (arg.type() === "String") {
      textarea.value = arg.string();
    } else {
      console.log("Invalid data received on /run/debug-app/ta-push:", arg);
    }
  });

  await p.subscribe(taTake, (arg) => {
    if (arg.type() !== "Func") {
      console.log("Invalid pointer received on /run/debug-app/ta-take:", arg);
      return;
    }
    p.publish(arg, FSObj.newString(textarea.value));
  });

  while (1) {
    await p.waitForEvent();
  }
}
