<script lang="ts">
  import { Session } from "../../../../wasm/cworks-wasm/pkg/cworks";
  import { Process } from "$lib/session";
  import Desktop from "$lib/components/Desktop.svelte";

  import { debug_main } from "./debug_app";
  import { lua_launcher } from "./lua_launcher";
  import { gui_main } from "./gui_app";

  function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  let sess: Session | null = null;
  let session_promise = sleep(100)
    .then(() => {
      let session = new Session();

      (async () => {
        while (1) {
          session.step();
          await new Promise((r) => setTimeout(r, 0));
        }
      })();

      sess = session;
      return session;
    })
    .then(async (sess) => {
      const session = sess;
      if (!session) {
        console.error("Session is not initialized");
        return;
      }

      await sleep(100);

      new Process(session, async (p) => {
        while (true) {
          await p.sleep(1);
        }
      });

      new Process(session, (p) => lua_launcher(p));
      new Process(session, (p) => gui_main(p));

      await sleep(100);

      new Process(session, (p) => debug_main(p));

      return sess;
    });
</script>

{#await session_promise}
  Waiting for kernel initialized...
{:then sess}
  <!-- <Xterm bind:terminal {options} {onLoad} {onData} {onKey} /> -->
  <Desktop />
{/await}
