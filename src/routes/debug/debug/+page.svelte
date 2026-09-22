<script lang="ts">
  import { Xterm, XtermAddon } from "@battlefieldduck/xterm-svelte";
  import type {
    ITerminalOptions,
    ITerminalInitOnlyOptions,
    Terminal,
  } from "@battlefieldduck/xterm-svelte";
  import { Session } from "../../../../wasm/cworks-wasm/pkg/cworks";
  import { Process } from "$lib/session";
  import Desktop from "$lib/components/Desktop.svelte";
  import { stdio_main } from "./stdio_app";
  import { debug_main } from "./debug_app";
  import { textarea_main } from "./textarea_app";
  import { lua_launcher } from "./lua_launcher";
  import { gui_main } from "./gui_app";

  function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  let code_editor: HTMLTextAreaElement | null = null;

  let terminal: Terminal;

  let options: ITerminalOptions & ITerminalInitOnlyOptions = {
    cursorStyle: "bar",
    cursorBlink: true,
    convertEol: true,
    theme: {
      cursor: "#cccccc",

      magenta: "#C678DD",
      green: "#98C379",
      background: "#1E2127",
      foreground: "#ABB2BF",
      black: "#1E2127",
      blue: "#61AFEF",
      cyan: "#56B6C2",
      red: "#E06C75",
      white: "#ABB2BF",
      yellow: "#E5C07B",
    },
    rows: 30,
  };

  let sess: Session | null = null;
  let session_promise = sleep(100).then(() => {
    let session = new Session();

    (async () => {
      while (1) {
        session.step();
        await new Promise((r) => setTimeout(r, 0));
      }
    })();

    sess = session;
    return session;
  });

  const canonicalize_newline = (data: string): string =>
    data.replace(/\r(\n)?/g, "\n");

  let input_buffer = "";
  let input_resolvers: Array<(key: string) => void> = [];
  function onData(data: string) {
    const key = canonicalize_newline(data);
    const resolve = input_resolvers.shift();
    if (resolve) {
      resolve(key);
    } else {
      input_buffer += key;
    }
  }

  function onKey(data: { key: string; domEvent: KeyboardEvent }) {}

  async function onLoad() {
    const fitAddon = new (await XtermAddon.FitAddon()).FitAddon();
    terminal.loadAddon(fitAddon);
    fitAddon.fit();

    const session = sess;
    if (!session) {
      console.error("Session is not initialized");
      return;
    }

    const stdio_process = new Process(session, (p) =>
      stdio_main(p, {
        stdin() {
          return new Promise((resolve) => {
            if (input_buffer.length > 0) {
              const buffered = input_buffer;
              input_buffer = "";
              resolve(buffered);
            } else {
              input_resolvers.push(resolve);
            }
          });
        },
        write(data) {
          terminal.write(canonicalize_newline(data));
        },
      }),
    );

    await sleep(100);

    const textarea_process = new Process(session, (p) =>
      textarea_main(p, code_editor as HTMLTextAreaElement),
    );

    const lua_launcher_process = new Process(session, (p) => lua_launcher(p));

    const gui_process = new Process(session, (p) => gui_main(p));

    await sleep(100);

    const debug_process = new Process(session, (p) => debug_main(p));
  }
</script>

<div class="relative">
  {#await session_promise}
    Waiting for kernel initialized...
  {:then sess}
    <Xterm bind:terminal {options} {onLoad} {onData} {onKey} />
  {/await}

  <Desktop />
</div>

<textarea bind:this={code_editor} rows="10" cols="50"> </textarea>
