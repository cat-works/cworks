// Shims for symbols wasm-bindgen's emscripten output references but does not
// declare (abort-handler globals and the HEAP32 accessor).

var __wbg_terminated_addr;
var __wbg_called_abort = false;

// wasm-bindgen declares its exported classes inside initBindgen but its
// module-scope shims reference them by name.
var FSObj, LuaEnv, Session;

function HEAP320() {
  return HEAP32;
}

function __wbg_call_abort_hook() {
  __wbg_called_abort = true;
  try {
    const idx = HEAP32[wasmExports.__abort_handler.value / 4];
    if (idx) {
      wasmExports.__indirect_function_table.get(idx)();
    }
  } catch (_) {}
}

function __wbg_handle_catch(e) {
  if (
    typeof WebAssembly.Exception !== "undefined" &&
    globalThis.__wbindgen_wrapped_jstag &&
    e instanceof WebAssembly.Exception &&
    e.is(globalThis.__wbindgen_wrapped_jstag)
  ) {
    throw e.getArg(globalThis.__wbindgen_wrapped_jstag, 0);
  }
  if (__wbg_terminated_addr === undefined) {
    __wbg_terminated_addr = wasmExports.__instance_terminated.value / 4;
  }
  HEAP32[__wbg_terminated_addr] = 1;
  __wbg_call_abort_hook();
  throw e;
}
