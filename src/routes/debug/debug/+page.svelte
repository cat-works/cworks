<script lang="ts">
  import { Xterm, XtermAddon } from "@battlefieldduck/xterm-svelte";
  import type {
    ITerminalOptions,
    ITerminalInitOnlyOptions,
    Terminal,
  } from "@battlefieldduck/xterm-svelte";
  import init, { Session } from "$lib/../wasm/pkg/wasm";
  import { FileSystem } from "$lib/fs_wrapper";
  import { writable } from "svelte/store";
  import { Process } from "$lib/session";
  import { stdio_main } from "./stdio_app";
  import { debug_main } from "./debug_app";

  function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  let terminal: Terminal;

  let options: ITerminalOptions & ITerminalInitOnlyOptions = {
    fontFamily: "Consolas",
    cursorStyle: "bar",
  };

  let sess: Session | null = null;
  let session_promise = init()
    .then(() => sleep(100))
    .then(() => {
      let session = new Session();

      let step_loop = setInterval(() => {
        try {
          session.step();
        } catch (e) {
          console.log("stepping failed");
          console.log("| reason =", e);
          clearInterval(step_loop);
        }
      }, 0);

      sess = session;
      return session;
    });

  let key_promise: ((key: string) => void) | null = null;
  function onData(data: string) {
    console.log(JSON.stringify(data), key_promise);
    if (key_promise) {
      key_promise(data);
      key_promise = null;
    } else {
      terminal.write("\x1b[2m" + data.replace("\r", "\r\n") + "\x1b[0m");
    }
  }

  function onKey(data: { key: string; domEvent: KeyboardEvent }) {
    console.log(data);
  }

  async function onLoad() {
    // FitAddon Usage
    const fitAddon = new (await XtermAddon.FitAddon()).FitAddon();
    terminal.loadAddon(fitAddon);
    fitAddon.fit();

    const session = sess;
    if (!session) {
      console.error("Session is not initialized");
      return;
    }

    const stdio_process = new Process((p) =>
      stdio_main(p, {
        stdin() {
          return new Promise((resolve) => {
            key_promise = (key: string) => {
              resolve(key.replace("\r", "\r\n"));
            };
          });
        },
        write(data) {
          terminal.write(data.replace("\n", "\r\n"));
        },
      })
    );
    session.add_process(stdio_process.kernel_callback.bind(stdio_process));

    await sleep(100);

    const debug_process = new Process(debug_main);
    session.add_process(debug_process.kernel_callback.bind(debug_process));
  }
</script>

{#await session_promise}
  Waiting for kernel initialized...
{:then sess}
  <Xterm bind:terminal {options} {onLoad} {onData} {onKey} />
{/await}
