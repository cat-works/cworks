// Pre-JS: callback registration system for cworks kernel integration
// This runs before the emscripten module initializes

// Callback registry: id -> function(data: string) -> string | null
Module._jsCallbacks = {};

// Register a callback function
Module._registerJsCallback = function(id, fn) {
  Module._jsCallbacks[id] = fn;
};

// Unregister a callback function
Module._unregisterJsCallback = function(id) {
  delete Module._jsCallbacks[id];
};

// Called from WASM via cworks-lib.js cworks_js_callback
Module._invokeJsCallback = function(id, data) {
  // Check for a specific callback first, then fall back to wildcard at ID 0
  var fn = Module._jsCallbacks[id] || Module._jsCallbacks[0];
  if (!fn) {
    console.error('cworks: no callback registered for id', id);
    return null;
  }
  try {
    return fn(id, data);
  } catch (e) {
    console.error('cworks: callback error for id', id, e);
    return null;
  }
};
