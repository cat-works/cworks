<script lang="ts">
  import { onDestroy, setContext } from "svelte";
  import { emitter, type WidgetState } from "$lib/desktop/state";
  import { HOST_KEY, hostFactory } from "$lib/desktop/host";
  import { renderers } from "$lib/desktop/renderers";
  import { Button } from "$lib/components/ui/button";
  import Widget from "./Widget.svelte";

  export let widget: WidgetState;

  const host = hostFactory.create(widget.obj);
  setContext(HOST_KEY, host);
  onDestroy(() => host.dispose());

  $: Renderer = renderers[widget.kind];

  let inputEl: HTMLInputElement | undefined;
  let lastValue: unknown;

  function trackTemplate(value: unknown, fallback: string): string {
    if (typeof value === "string") {
      return value;
    }
    const count = Number(value) || 1;
    return `repeat(${count},minmax(0,1fr))`;
  }

  function gridStyle(props: Record<string, unknown>): string {
    const gap = Number(props.gap) || 0;
    let style =
      `display:grid;` +
      `grid-template-columns:${trackTemplate(props.columns, "1fr")};` +
      `gap:${gap}px;`;
    if (props.rows !== undefined) {
      style += `grid-template-rows:${trackTemplate(props.rows, "1fr")};height:100%;`;
    }
    return style;
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

  function dragOf(props: Record<string, unknown>): string | undefined {
    return typeof props.drag === "string" ? props.drag : undefined;
  }

  function boxStyle(props: Record<string, unknown>): string {
    let style = "";
    if (typeof props.background === "string") {
      style += `background:${props.background};`;
    }
    if (typeof props.border === "string") {
      style += `border:${props.border};`;
    }
    if (typeof props.color === "string") {
      style += `color:${props.color};`;
    }
    if (props.padding !== undefined) {
      style += `padding:${Number(props.padding)}px;`;
    }
    return style;
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
  <div style={gridStyle(widget.props) + boxStyle(widget.props)}>
    {#each widget.children as child (child.id)}
      <div
        style="{areaStyle(child.props)}{dragOf(child.props)
          ? 'touch-action:none;user-select:none;'
          : ''}"
        data-widget-id={child.id}
        data-drag={dragOf(child.props)}
      >
        <Widget widget={child} />
      </div>
    {/each}
  </div>
{:else if widget.kind === "label"}
  <span style={boxStyle(widget.props)}>{widget.props.text}</span>
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
    style={boxStyle(widget.props)}
    on:input={onInput}
  />
{:else if widget.kind === "html"}
  <div style={boxStyle(widget.props)}>{@html String(widget.props.html ?? "")}</div>
{:else if Renderer}
  <Renderer {widget} />
{/if}
