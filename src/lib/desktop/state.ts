import { writable } from "svelte/store";

export interface WidgetState {
  id: number;
  name: string;
  kind: string;
  props: Record<string, unknown>;
  children: WidgetState[];
}

export interface WindowState {
  id: number;
  title: string;
  options: string;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;
  content: WidgetState | null;
}

export const desktop = writable<WindowState[]>([]);

export const emitter: {
  send: (widgetId: number, event: string, value?: string) => void;
} = {
  send: () => {},
};
