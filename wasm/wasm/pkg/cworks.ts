import cworks_mod from "./cworks-rs";

const mod = await cworks_mod();
mod.ccall("__ffi_init", null, [], []);

// ─── Symbol demangling ───

export function demangle_str(x: string): string {
  return mod.ccall("__ffi_demangle", "string", ["string"], [x]);
}

// ─── Lua Thread (existing API, backward-compatible) ───

export class LuaThreadError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "LuaThreadError";
  }
}

export class LuaEnv {
  ptr: number;
  constructor() {
    this.ptr = mod.___ffi_lufenv_new();
  }

  run(code: string) {
    mod.ccall("__ffi_luaenv_run", null, ["number", "string"], [this.ptr, code]);
  }
}

export function createNativeLuaProcess(env: LuaEnv, name: string, code: string) {
  const success = mod.ccall(
    "__ffi_lua_process_new",
    "number",
    ["number", "string", "string"],
    [env.ptr, name, code],
  );

  console.log("NativeLuaProcess created with PID:", success);
  return BigInt(success);
}

// ─── Kernel Session (new unified API) ───

export type KernelCallback = (data: any) => any;

export class Session {
  private nextCallbackId = 1;
  private callbacks = new Map<number, KernelCallback>();

  constructor() {
    mod.___ffi_session_new();
    // Register the global dispatcher that Rust will call via cworks_js_callback.
    // cworks_js_callback(id, dataPtr, dataLen) → Module._invokeJsCallback(id, data)
    (mod as any)._registerJsCallback(0, (id: number, data: string) => {
      return this.dispatchCallback(id, data);
    });
  }

  /**
   * Register a callback function that will be invoked when the kernel
   * polls a process. Returns a numeric callback ID to pass to Rust.
   */
  registerCallback(fn: KernelCallback): number {
    const id = this.nextCallbackId++;
    this.callbacks.set(id, fn);
    return id;
  }

  /**
   * Unregister a previously registered callback.
   */
  unregisterCallback(id: number) {
    this.callbacks.delete(id);
  }

  /**
   * Add a process to the kernel using a raw callback function.
   * This is the backward-compatible API matching the old wasm Session.
   */
  add_process(callback: Function): bigint {
    const id = this.registerCallback(callback as KernelCallback);
    return this.addProcess(id);
  }

  /**
   * Add a process to the kernel.
   * `callbackId` is the ID returned by `registerCallback()`.
   * Returns the process PID.
   */
  addProcess(callbackId: number): bigint {
    const result = mod.___ffi_session_add_process(callbackId);
    return BigInt(result as unknown as number);
  }

  /**
   * Advance the kernel by one tick. This triggers polling of all
   * processes, which in turn invoke their registered JS callbacks.
   */
  step() {
    mod.___ffi_session_step();
  }

  /**
   * Internal: dispatcher called from Rust via cworks_js_callback.
   * Routes the callback to the appropriate registered function.
   */
  private dispatchCallback(id: number, data: string): string {
    const fn = this.callbacks.get(id);
    if (!fn) {
      console.error(`cworks: no callback registered for id ${id}`);
      return JSON.stringify(null);
    }
    try {
      // Rust wraps data as {"id":callback_id,"data":<SyscallData JSON>}
      const { id: _routedId, data: syscallData } = JSON.parse(data);
      const result = fn(syscallData);
      // Serialize BigInt as JSON numbers (u128 fields like WaitForProcess
      // payloads must deserialize as numbers on the Rust side).
      return JSON.stringify(result, (_key, value) =>
        typeof value === "bigint" ? Number(value) : value
      );
    } catch (e) {
      console.error(`cworks: callback error for id ${id}`, e);
      return JSON.stringify(null);
    }
  }
}
