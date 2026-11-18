import type { Process } from "$lib/session";
import {
  background,
  desktop,
  emitter,
  type WidgetState,
  type WindowState,
} from "$lib/desktop/state";
import { hostFactory, type RendererHost } from "$lib/desktop/host";
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
    const value = obj.getObj(child);
    if (value.type() === "Func") {
      continue;
    }
    props[child] = readValue(value);
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
  return { id: obj.id(), name, kind: obj.getObj("kind").string(), obj, props, children };
}

function readFrame(
  obj: FSObj,
  controls: Map<number, FSObj>,
  events: Map<number, FSObj>,
): WindowState {
  const frame = readWidget(obj, "frame", controls, events);
  return {
    obj,
    x: Number(frame.props.x ?? 40),
    y: Number(frame.props.y ?? 40),
    w: Number(frame.props.w ?? 360),
    h: Number(frame.props.h ?? 260),
    z: Number(frame.props.z ?? 1),
    frame,
  };
}

export async function gui_main(p: Process): Promise<void> {
  const root = await p.fsRoot();
  const guiNs = root.ensureCompound("run").ensureCompound("gui");
  const appReq = guiNs.ensureFunc("req");
  const wmReq = guiNs.ensureFunc("wm-req");
  const wmRes = guiNs.ensureFunc("wm-res");

  const display = guiNs.ensureCompound("display");
  display.ensureString("kind", "display");
  const displayControl = display.ensureFunc("control");
  display.ensureFunc("event");

  const subscribedControls = new Map<number, {obj: FSObj, unsubscribe: () => void}>();
  const pendingCreates: FSObj[] = [];
  let wmReady = false;
  let eventHandles = new Map<number, FSObj>();

  hostFactory.create = (obj: FSObj): RendererHost => {
    const subscriptions = new Map<string, { obj: FSObj, unsubscribe: () => void }>();
    return {
      obj,
      subscribe(channel: string, handler: (arg: FSObj) => void): void {
        const handle = obj.getObj(channel);
        subscriptions.set(channel, {
          obj: handle,
          unsubscribe: handle.subscribe(handler)
        });
      },
      unsubscribe(channel: string): void {
        const handle = subscriptions.get(channel);
        if (handle) {
          handle.unsubscribe();
          subscriptions.delete(channel);
        }
      },
      publishString(channel: string, value: string): void {
        obj.getObj(channel).publish(FSObj.newString(value));
      },
      emit(event: string, value?: string): void {
        const msg = FSObj.newCompound();
        msg.createString("event", event);
        if (value !== undefined) {
          msg.createString("value", value);
        }
        void obj.getObj("event").publish(msg);
      },
      dispose(): void {
        for (const [channel, handle] of subscriptions) {
          subscriptions.delete(channel);
          void handle.unsubscribe();
        }
      },
    };
  };

  function attachDisplay(): void {
    const msg = FSObj.newCompound();
    msg.createString("op", "attach");
    msg.addChild("display", display);
    void wmReq.publish(msg);
  }

  function wmSend(op: string, window: FSObj, extra?: Record<string, number>): void {
    const msg = FSObj.newCompound();
    msg.createString("op", op);
    msg.addChild("window", window);
    if (extra) {
      for (const [key, value] of Object.entries(extra)) {
        msg.createInt(key, value);
      }
    }
    void wmReq.publish(msg);
  }

  async function reconcile(recheck = true): Promise<void> {
    const controls = new Map<number, FSObj>();
    const events = new Map<number, FSObj>();
    const windows: WindowState[] = [];

    for (const name of display.list()) {
      if (RESERVED.has(name)) {
        continue;
      }
      const child = display.getObj(name);
      if (child.type() !== "CompoundFSObj" || !child.hasChild("kind")) {
        continue;
      }
      windows.push(readFrame(child, controls, events));
    }
    windows.sort((a, b) => a.z - b.z);

    for (const [id, obj] of [...subscribedControls]) {
      if (!controls.has(id)) {
        subscribedControls.delete(id);
        obj.unsubscribe();
      }
    }

    const newSubscriptions: Array<[number, {obj: FSObj, unsubscribe: ()=>void}]> = [];
    for (const [id, obj] of controls) {
      if (!subscribedControls.has(id)) {
        const unsubscribe = obj.subscribe(obj => reconcile())
        subscribedControls.set(id, {obj, unsubscribe});
        newSubscriptions.push([id, {obj, unsubscribe}]);
      }
    }

    eventHandles = events;
    desktop.set(windows);

    if (display.hasChild("background")) {
      background.set(display.getObj("background").string());
    }

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
    void handle.publish(msg);
  };
  emitter.focus = (window) => wmSend("focus", window);
  emitter.move = (window, x, y) => wmSend("move", window, { x, y });
  emitter.resize = (window, w, h) => wmSend("resize", window, { w, h });

  displayControl.subscribe(() => void reconcile());

  appReq.subscribe((arg) => {
    if (wmReady) {
      wmReq.publish(arg);
    } else {
      pendingCreates.push(arg);
    }
  });

  wmRes.subscribe((arg) => {
    if (arg.type() !== "CompoundFSObj" || !arg.hasChild("op")) {
      return;
    }
    const op = arg.getObj("op").string();
    if (op === "sync") {
      attachDisplay();
    } else if (op === "ready") {
      wmReady = true;
      for (const create of pendingCreates.splice(0)) {
        void wmReq.publish(create);
      }
    }
  });

  attachDisplay();

  // Keep the process Running (rather than blocking in waitForEvent) so that
  // requests queued from DOM events (emitter.*) are pumped on the next step.
  await new Promise<void>(() => { });
}
