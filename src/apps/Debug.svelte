<script lang="ts">
  import { sleep } from "../libs/utils";
  import { Process } from "../libs/session";
  import init, { Session } from "../wasm/pkg/wasm";
  import ConsoleLogger from "@src/components/ConsoleLogger.svelte";
  import { FileSystem } from "../libs/fs_wrapper";

  let kernel = init()
    .then(() => sleep(100))
    .then(() => {
      let session = new Session();

      let p2 = new Process(async (p: Process) => {
        await p.sleep(0.1);

        console.log("Waiting FS IPC...");

        let fs = new FileSystem(p);
        await fs.wait_for_ready();

        console.log("FS Connected");
        try {
          await fs.cd("usr/mime/cafecode");
          await fs.root();
          await fs.cd("usr/mime/cafecode");
          console.log(await fs.list());
          console.log(await fs.get("text"));
          await fs.set_raw("text", "String?neko");
          console.log(await fs.get("text"));
        } catch (error) {
          console.log("error", error);
        }

        return 0n;
      });

      session.add_process(p2.kernel_callback.bind(p2));

      console.log("Starting session stepping");
      let step_loop = setInterval(() => {
        try {
          session.step();
        } catch (e) {
          console.log("stepping failed");
          console.log("| reason =", e);
          clearInterval(step_loop);
        }
      }, 0);
    });
</script>

{#await kernel}
  Initalizing kernel...
{:then _}
  Kernel: Ok
{/await}

<ConsoleLogger />
