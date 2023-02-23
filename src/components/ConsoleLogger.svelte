<script lang="ts">
  import { my_stringify, timestamp } from "@src/libs/utils";
  import { writable } from "svelte/store";

  let buf = writable("");

  eval("console.clog = console.log");

  let clog = console.log;
  window.console.log = (...args) => {
    clog(...args);
    const time = timestamp();
    const data = args[0].toString().includes("%c")
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
    clog(data);
    buf.update((x) => `${x}\n${time} ${data}`);
  };
  window.console.info = (...args) => window.console.log(...args);
  window.console.warn = (...args) => window.console.log(...args);
  window.console.error = (...args) => window.console.log(...args);
  window.console.debug = (...args) => window.console.log(...args);
  window.console.trace = (...args) => window.console.log(...args);
</script>

<pre><div
    style="font-family: 'Fira Code'; width: 100%;word-wrap: break-word;">{@html $buf}</div></pre>
