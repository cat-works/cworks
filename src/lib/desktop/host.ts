import { getContext } from "svelte";
import type { FSObj } from "../../../wasm/cworks-wasm/pkg/cworks";

/**
 * Per-widget host shared with hosted renderers via Svelte context. It exposes
 * the widget's raw FSObj plus channel operations implemented by the display
 * server, so renderers never touch the kernel API directly.
 */
export interface RendererHost {
  obj: FSObj;
  subscribe(channel: string, handler: (arg: FSObj) => void): void;
  unsubscribe(channel: string): void;
  publishString(channel: string, value: string): void;
  emit(event: string, value?: string): void;
  dispose(): void;
}

export const HOST_KEY = Symbol("cworks:renderer-host");

export const hostFactory: {
  create: (obj: FSObj) => RendererHost;
} = {
  create: () => ({
    obj: undefined as unknown as FSObj,
    subscribe: () => {},
    unsubscribe: () => {},
    publishString: () => {},
    emit: () => {},
    dispose: () => {},
  }),
};

export function getRendererHost(): RendererHost {
  return getContext<RendererHost>(HOST_KEY);
}
