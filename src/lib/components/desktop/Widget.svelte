<script lang="ts">
  import { emitter, type WidgetState } from "$lib/desktop/state";
  import { Button } from "$lib/components/ui/button";
  import Widget from "./Widget.svelte";

  export let widget: WidgetState;

  let inputEl: HTMLInputElement | undefined;
  let lastValue: unknown;

  function gridStyle(props: Record<string, unknown>): string {
    const columns = Number(props.columns) || 1;
    const gap = Number(props.gap) || 0;
    return `display:grid;grid-template-columns:repeat(${columns},minmax(0,1fr));gap:${gap}px;`;
  }

  function areaStyle(props: Record<string, unknown>): string {
    const area = props.area;
    if (typeof area !== "string") {
      return "";
    }
    const match = area.match(/^(\d+),(\d+)-(\d+),(\d+)$/);
    if (!match) {
      return "";
    }
    const r1 = Number(match[1]);
    const c1 = Number(match[2]);
    const r2 = Number(match[3]);
    const c2 = Number(match[4]);
    return `grid-row:${r1 + 1} / ${r2 + 1};grid-column:${c1 + 1} / ${c2 + 1};`;
  }

  function onInput(event: Event): void {
    emitter.send(widget.id, "input", (event.currentTarget as HTMLInputElement).value);
  }

  $: if (inputEl) {
    const value = widget.props.value == null ? "" : String(widget.props.value);
    if (value !== lastValue) {
      inputEl.value = value;
      lastValue = value;
    }
  }
</script>

{#if widget.kind === "grid"}
  <div style={gridStyle(widget.props)}>
    {#each widget.children as child (child.id)}
      <div style={areaStyle(child.props)}>
        <Widget widget={child} />
      </div>
    {/each}
  </div>
{:else if widget.kind === "label"}
  <span>{widget.props.text}</span>
{:else if widget.kind === "button"}
  <Button size="sm" on:click={() => emitter.send(widget.id, "click")}>
    {widget.props.text}
  </Button>
{:else if widget.kind === "input"}
  <input
    bind:this={inputEl}
    type={String(widget.props.type ?? "text")}
    placeholder={widget.props.placeholder == null
      ? ""
      : String(widget.props.placeholder)}
    class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
    on:input={onInput}
  />
{:else if widget.kind === "html"}
  <div>{@html String(widget.props.html ?? "")}</div>
{/if}
