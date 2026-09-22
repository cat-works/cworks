<script lang="ts">
  import { desktop, emitter, type WindowState } from "$lib/desktop/state";
  import Widget from "./Widget.svelte";

  export let win: WindowState;

  function focus(): void {
    desktop.update((windows) => {
      const max = windows.reduce((m, w) => Math.max(m, w.z), 0);
      return windows.map((w) => (w.id === win.id ? { ...w, z: max + 1 } : w));
    });
  }

  function close(event: MouseEvent): void {
    event.stopPropagation();
    emitter.send(win.id, "close");
  }
</script>

<div
  class="pointer-events-auto absolute flex flex-col overflow-hidden rounded-lg border bg-card text-card-foreground shadow-lg"
  style="left:{win.x}px; top:{win.y}px; width:{win.w}px; height:{win.h}px; z-index:{win.z};"
  on:mousedown={focus}
  role="presentation"
>
  <div class="flex h-8 shrink-0 items-center justify-between border-b bg-muted px-2">
    <span class="truncate text-xs font-medium">{win.title}</span>
    <button
      class="rounded px-1 text-sm leading-none hover:bg-accent"
      on:click={close}
      aria-label="Close">×</button
    >
  </div>
  <div class="min-h-0 flex-1 overflow-auto p-2">
    {#if win.content}
      <Widget widget={win.content} />
    {/if}
  </div>
</div>
