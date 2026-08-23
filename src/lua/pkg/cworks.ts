import cworks_mod from "./cworks-rs";

const mod = await cworks_mod();
mod.ccall("__ffi_init", null, [], []);

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

  thread(name: string, code: string): LuaThread {
    const ptr = mod.ccall(
      "__ffi_luaenv_thread",
      "number",
      ["number", "string", "string"],
      [this.ptr, name, code],
    );
    return new LuaThread(ptr);
  }
}

export class LuaThread {
  ptr: number;

  constructor(ptr: number) {
    this.ptr = ptr;
  }

  yield(arg: string): string {
    const encoded_arg = encode(arg);
    const encoded_value = mod.ccall(
      "__ffi_lua_thread_yield",
      "string",
      ["number", "string"],
      [this.ptr, encoded_arg],
    );
    if (encoded_value === "\x01\x03") {
      throw new LuaThreadError("Lua thread encountered an error.");
    }
    return decode(encoded_value);
  }
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

// ─── Encoding helpers (NUL-byte safe) ───

function encode(input: string): string {
  let result = "";
  for (const ch of input) {
    if (ch === "\0") {
      result += "\x01\x02";
    } else if (ch === "\x01") {
      result += "\x01\x01";
    } else {
      result += ch;
    }
  }
  return result;
}

function decode(input: string): string {
  let result = "";
  for (let i = 0; i < input.length; i++) {
    if (input[i] === "\x01") {
      if (input[i + 1] === "\x02") {
        result += "\0";
        i++;
      } else if (input[i + 1] === "\x01") {
        result += "\x01";
        i++;
      } else {
        result += "\x01";
      }
    } else {
      result += input[i];
    }
  }
  return result;
}
