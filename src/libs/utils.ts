import { writable } from "svelte/store";

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function my_stringify(x: object) {
  return JSON.stringify(x, (_, v) =>
    typeof v === "bigint" ? v.toString() : v
  );
}
eval("window.my_stringify = my_stringify");

export function timestamp(): string {
  return new Date().toISOString();
}

export function patch_console_log() {
  let buf = writable("");

  eval("console.clog = console.log");

  let clog = console.log;
  window.console.log = (...args) => {
    clog(...args);
    const time = timestamp();
    const data = args
      .map((arg) => {
        if (!arg) {
          return "null";
        }

        if (typeof arg === "string") {
          return arg;
        }

        return my_stringify(arg);
      })
      .join(" ");

    buf.update(x => `${x}\n${time} ${data}`);
  };

  return buf;
}