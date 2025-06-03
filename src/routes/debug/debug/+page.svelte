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
  import { textarea_main } from "./textarea_app";

  function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  let code_editor: HTMLTextAreaElement | null = null;

  let terminal: Terminal;

  let options: ITerminalOptions & ITerminalInitOnlyOptions = {
    fontFamily: "Consolas",
    cursorStyle: "bar",
    cursorBlink: true,
    convertEol: true,
    theme: {
      background: "#000000",
      foreground: "#ffffff",
      cursor: "#ffffff",
      black: "#000000",
      red: "#ff0000",
      green: "green",
      yellow: "#ffff00",
      blue: "#0000ff",
      magenta: "#ff00ff",
      cyan: "#00ffff",
      white: "#ffffff",
      brightRed: "#ff0000",
    },
  };

  let sess: Session | null = null;
  let session_promise = init()
    .then(() => sleep(100))
    .then(() => {
      let session = new Session();

      (async () => {
        while (1) {
          try {
            session.step();
          } catch (e) {
            debugger;
            throw e;
          }
          await new Promise((r) => setTimeout(r, 0));
        }
      })();

      sess = session;
      return session;
    });

  const canonicalize_newline = (data: string): string =>
    data.replace(/\r(\n)?/g, "\n");

  let key_promise: ((key: string) => void) | null = null;
  function onData(data: string) {
    if (key_promise) {
      key_promise(data);
      key_promise = null;
    } else {
      terminal.write("\x1b[2m" + canonicalize_newline(data) + "\x1b[0m");
    }
  }

  function onKey(data: { key: string; domEvent: KeyboardEvent }) {}

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
              resolve(canonicalize_newline(key));
            };
          });
        },
        write(data) {
          terminal.write(canonicalize_newline(data));
        },
      })
    );
    session.add_process(stdio_process.kernel_callback.bind(stdio_process));

    const textarea_process = new Process((p) =>
      textarea_main(p, code_editor as HTMLTextAreaElement)
    );
    session.add_process(
      textarea_process.kernel_callback.bind(textarea_process)
    );

    await sleep(100);

    const debug_process = new Process((p) => debug_main(p, sess as Session));
    session.add_process(debug_process.kernel_callback.bind(debug_process));
  }
</script>

{#await session_promise}
  Waiting for kernel initialized...
{:then sess}
  <Xterm bind:terminal {options} {onLoad} {onData} {onKey} />
{/await}

<textarea bind:this={code_editor} rows="10" cols="50"> </textarea>
