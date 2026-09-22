import { writable } from "svelte/store";
import type { FSObj } from "../../../wasm/cworks-wasm/pkg/cworks";

export interface WidgetState {
  id: number;
  name: string;
  kind: string;
  props: Record<string, unknown>;
  children: WidgetState[];
}

export interface WindowState {
  /** Frame root FSObj (a child of the Display). Used for window-manager messages. */
  obj: FSObj;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;
  /** The frame widget tree (WM-built decoration + app content). */
  frame: WidgetState;
}

export const desktop = writable<WindowState[]>([]);

// Requests sent to the window manager process (see gui_app.ts / wm.lua).
export const emitter: {
  send: (widgetId: number, event: string, value?: string) => void;
  focus: (window: FSObj) => void;
  move: (window: FSObj, x: number, y: number) => void;
  resize: (window: FSObj, w: number, h: number) => void;
} = {
  send: () => {},
  focus: () => {},
  move: () => {},
  resize: () => {},
};
