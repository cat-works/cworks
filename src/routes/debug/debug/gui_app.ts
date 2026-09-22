import type { Process } from "$lib/session";
import {
  desktop,
  emitter,
  type WidgetState,
  type WindowState,
} from "$lib/desktop/state";
import { FSObj } from "../../../../wasm/cworks-wasm/pkg/cworks";

const RESERVED = new Set(["kind", "control", "event", "content", ".", ".."]);

function readValue(obj: FSObj): unknown {
  switch (obj.type()) {
    case "String":
      return obj.string();
    case "Int":
      return obj.int();
    case "Boolean":
      return obj.boolean();
    case "Float":
      return obj.float();
    case "Double":
      return obj.double();
    default:
      return null;
  }
}

function readWidget(
  obj: FSObj,
  name: string,
  controls: Map<number, FSObj>,
  events: Map<number, FSObj>,
): WidgetState {
  const props: Record<string, unknown> = {};
  for (const child of obj.list()) {
    if (RESERVED.has(child)) {
      continue;
    }
    props[child] = readValue(obj.getObj(child));
  }

  const children: WidgetState[] = [];
  if (obj.hasChild("content")) {
    const content = obj.getObj("content");
    for (const child of content.list()) {
      if (child === "." || child === "..") {
        continue;
      }
      children.push(readWidget(content.getObj(child), child, controls, events));
    }
  }

  controls.set(obj.id(), obj.getObj("control"));
  events.set(obj.id(), obj.getObj("event"));
  return { id: obj.id(), name, kind: obj.getObj("kind").string(), props, children };
}

function readWindow(
  obj: FSObj,
  cascade: number,
  controls: Map<number, FSObj>,
  events: Map<number, FSObj>,
  z: number,
): WindowState {
  const prop = (key: string, fallback: unknown): unknown =>
    obj.hasChild(key) ? readValue(obj.getObj(key)) : fallback;

  let root: WidgetState | null = null;
  const content = obj.getObj("content");
  for (const child of content.list()) {
    if (child === "." || child === "..") {
      continue;
    }
    root = readWidget(content.getObj(child), child, controls, events);
    break;
  }

  controls.set(obj.id(), obj.getObj("control"));
  events.set(obj.id(), obj.getObj("event"));
  return {
    id: obj.id(),
    title: String(prop("title", "Window")),
    options: String(prop("options", "")),
    x: Number(prop("x", 40 + cascade * 24)),
    y: Number(prop("y", 40 + cascade * 24)),
    w: Number(prop("w", 360)),
    h: Number(prop("h", 260)),
    z,
    content: root,
  };
}

export async function gui_main(p: Process): Promise<void> {
  const root = await p.fsRoot();
  const control = root.ensureCompound("run").ensureCompound("gui").ensureFunc("req");

  const windowObjs: FSObj[] = [];
  const subscribedControls = new Map<number, FSObj>();
  const zOrder = new Map<number, number>();
  let eventHandles = new Map<number, FSObj>();
  let nextZ = 1;

  function controlHandler(arg: FSObj): void {
    if (arg.type() === "CompoundFSObj" && arg.hasChild("op")) {
      const op = arg.getObj("op").string();
      if (op === "close" && arg.hasChild("node")) {
        const id = arg.getObj("node").id();
        const index = windowObjs.findIndex((w) => w.id() === id);
        if (index >= 0) {
          windowObjs.splice(index, 1);
        }
      }
    }
    void reconcile();
  }

  async function reconcile(recheck = true): Promise<void> {
    const controls = new Map<number, FSObj>();
    const events = new Map<number, FSObj>();
    const windows: WindowState[] = [];

    windowObjs.forEach((obj, index) => {
      const id = obj.id();
      if (!zOrder.has(id)) {
        zOrder.set(id, nextZ++);
      }
      windows.push(readWindow(obj, index, controls, events, zOrder.get(id)!));
    });
    windows.sort((a, b) => a.z - b.z);

    for (const [id, obj] of [...subscribedControls]) {
      if (!controls.has(id)) {
        subscribedControls.delete(id);
        void p.unsubscribe(obj);
      }
    }

    const newSubscriptions: Array<[number, FSObj]> = [];
    for (const [id, obj] of controls) {
      if (!subscribedControls.has(id)) {
        subscribedControls.set(id, obj);
        newSubscriptions.push([id, obj]);
      }
    }
    await Promise.all(
      newSubscriptions.map(([, obj]) => p.subscribe(obj, controlHandler)),
    );

    eventHandles = events;
    desktop.set(windows);

    if (newSubscriptions.length > 0 && recheck) {
      await reconcile(false);
    }
  }

  emitter.send = (widgetId: number, event: string, value?: string): void => {
    const handle = eventHandles.get(widgetId);
    if (!handle) {
      return;
    }
    const msg = FSObj.newCompound();
    msg.createString("event", event);
    if (value !== undefined) {
      msg.createString("value", value);
    }
    void p.publish(handle, msg);
  };

  await p.subscribe(control, (arg) => {
    windowObjs.push(arg);
    void reconcile();
  });

  // Keep the process Running (rather than blocking in waitForEvent) so that
  // requests queued from DOM events (emitter.send) are pumped on the next step.
  await new Promise<void>(() => {});
}
