<script lang="ts">
  import { desktop } from "$lib/desktop/state";
  import Window from "./desktop/Window.svelte";
</script>

<div class="pointer-events-none absolute inset-0 z-20 overflow-hidden">
  {#each $desktop as win (win.obj.id())}
    <Window {win} />
  {/each}
</div>
