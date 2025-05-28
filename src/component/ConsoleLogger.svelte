<script lang="ts">
  import { my_stringify, timestamp } from "$lib/console-utils";
  import { writable } from "svelte/store";

  let buf = writable("");

  eval("console.clog = console.log");

  let clog = console.log;
  let f = (...args) => {
    clog(...args);
    const time = timestamp();
    const _data = args[0].toString().includes("%c")
      ? [...args.shift().match(/(%C)?[^%]*/gi)]
          .map((x) =>
            x.match(/^%C/i)
              ? `<span style="${args.shift()}">${x.slice(2)}</span>`
              : x
          )
          .join(" ")
      : args
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
    const data = _data
      .replace("kernel/src/kernel_processes/mod.rs", "[KernelP]")
      .replace("kernel/src/kernel.rs", "[Kernel]")
      .replace("wasm/src/lib.rs", "[wasm]");
    clog("a");
    buf.update((x) => `${x}\n${time} ${data}`);
  };
  globalThis.console.log = (...args) => f(...args);
  globalThis.console.info = (...args) => f(...args);
  globalThis.console.warn = (...args) => f(...args);
  globalThis.console.error = (...args) => f(...args);
  globalThis.console.debug = (...args) => f(...args);
  globalThis.console.trace = (...args) => f(...args);
</script>

<div class="container">
  <span class="header">console.xxx</span>
  <pre class="datas"><div
      style="font-family: 'Fira Code'; width: 100%;word-wrap: break-word;">{@html $buf}</div></pre>
</div>

<style>
  .container {
    position: relative;

    background-color: #2224;
    border: 1px solid #333;
  }
  .datas {
    margin-top: 1rem;
  }
  .header {
    position: absolute;
    top: 0;
    left: 0;
    background-color: #3334;

    border-radius: 0.5rem 0.5rem 0 0;
  }
</style>
