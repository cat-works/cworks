import type { Component } from "svelte";
import type { WidgetState } from "$lib/desktop/state";
import XtermRenderer from "$lib/components/desktop/XtermRenderer.svelte";

/**
 * Hosted renderers resolve widget kinds that the display server does not
 * implement as built-in primitives. Adding a renderer does not change the
 * widget protocol.
 */
export const renderers: Record<string, Component<{ widget: WidgetState }>> = {
  terminal: XtermRenderer,
};
