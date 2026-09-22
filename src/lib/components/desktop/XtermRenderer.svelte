<script lang="ts">
  import { onDestroy } from "svelte";
  import { Xterm, XtermAddon } from "@battlefieldduck/xterm-svelte";
  import type { ITerminalInitOnlyOptions, ITerminalOptions, Terminal } from "@xterm/xterm";
  import { getRendererHost } from "$lib/desktop/host";
  import type { WidgetState } from "$lib/desktop/state";

  export let widget: WidgetState;

  const host = getRendererHost();

  let terminal: Terminal | undefined;
  let fitAddon: { fit: () => void } | undefined;

  const options: ITerminalOptions & ITerminalInitOnlyOptions = {
    cursorStyle: "bar",
    cursorBlink: true,
    convertEol: true,
    fontSize: Number(widget.props.fontSize) || 13,
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
  };

  async function onLoad(term: Terminal): Promise<void> {
    terminal = term;
    const mod = await XtermAddon.FitAddon();
    fitAddon = new mod.FitAddon();
    term.loadAddon(fitAddon as never);
    requestAnimationFrame(() => {
      try {
        fitAddon?.fit();
      } catch {
        /* the container may not be laid out yet */
      }
    });
    host.subscribe("output", (arg) => {
      if (arg.type() === "String") {
        term.write(arg.string());
      }
    });
    host.emit("ready");
  }

  function onData(data: string): void {
    // The process does not echo, so mirror the input locally (as the page
    // terminal does): Enter advances the line, Backspace erases a character.
    if (terminal) {
      for (const ch of data) {
        if (ch === "\r") {
          terminal.write("\n");
        } else if (ch === "\x7f") {
          terminal.write("\b \b");
        } else {
          terminal.write(ch);
        }
      }
    }
    host.publishString("input", data);
  }

  onDestroy(() => {
    terminal?.dispose();
  });
</script>

<Xterm {options} onLoad={onLoad} onData={onData} style="height:100%;width:100%;" />
