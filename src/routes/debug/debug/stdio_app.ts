import type { Process } from "$lib/session";
import { FSObj } from "../../../../wasm/cworks-wasm/pkg/cworks";

export async function stdio_main(
  p: Process,
  terminal: { stdin: () => Promise<string>; write: (data: string) => void },
) {
  const root = await p.fsRoot();
  const debugApp = root.ensureCompound("run").ensureCompound("debug-app");
  const shellOut = debugApp.ensureFunc("shell-out");
  const shellIn = debugApp.ensureFunc("shell-in");

  await p.subscribe(shellOut, (arg) => {
    if (arg.type() === "String") {
      terminal.write(arg.string());
    } else {
      console.error("Invalid data received on /run/debug-app/shell-out:", arg);
    }
  });

  while (1) {
    const key = await terminal.stdin();
    terminal.write(key);
    await p.publish(shellIn, FSObj.newString(key));
  }
}
