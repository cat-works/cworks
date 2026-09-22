<script lang="ts">
  import { emitter, type WindowState } from "$lib/desktop/state";
  import Widget from "./Widget.svelte";

  export let win: WindowState;

  function onPointerDown(event: PointerEvent): void {
    emitter.focus(win.obj);

    const target = event.target as HTMLElement | null;
    const handle = target?.closest("[data-drag]") as HTMLElement | null;
    if (!handle) {
      return;
    }
    const mode = handle.dataset.drag;
    if (mode !== "move" && mode !== "resize") {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const startX = event.clientX;
    const startY = event.clientY;
    const origX = win.x;
    const origY = win.y;
    const origW = win.w;
    const origH = win.h;

    let pending: { a: number; b: number } | null = null;
    let raf = 0;

    const flush = () => {
      raf = 0;
      if (!pending) {
        return;
      }
      if (mode === "move") {
        emitter.move(win.obj, pending.a, pending.b);
      } else {
        emitter.resize(win.obj, pending.a, pending.b);
      }
      pending = null;
    };

    const onMove = (ev: PointerEvent) => {
      const dx = ev.clientX - startX;
      const dy = ev.clientY - startY;
      if (mode === "move") {
        pending = { a: origX + dx, b: origY + dy };
      } else {
        pending = { a: Math.max(80, origW + dx), b: Math.max(60, origH + dy) };
      }
      if (!raf) {
        raf = requestAnimationFrame(flush);
      }
    };

    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      if (raf) {
        cancelAnimationFrame(raf);
        raf = 0;
      }
      flush();
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }
</script>

<div
  class="pointer-events-auto absolute overflow-hidden"
  style="left:{win.x}px; top:{win.y}px; width:{win.w}px; height:{win.h}px; z-index:{win.z};"
  on:pointerdown={onPointerDown}
  role="presentation"
>
  <Widget widget={win.frame} />
</div>
