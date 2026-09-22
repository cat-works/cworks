#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
wasm_dir=$(dirname "$here")
out="$here/pkg"

mkdir -p "$out"
cd "$wasm_dir"

cargo build -p cworks-wasm --target wasm32-unknown-emscripten --release

emcc --no-entry "target/wasm32-unknown-emscripten/release/libcworks_wasm.a" \
  -sWASM_BINDGEN \
  -fwasm-exceptions \
  --pre-js="$here/pre.js" \
  -sMODULARIZE=1 \
  -sEXPORT_ES6=1 \
  -sWASM_BIGINT \
  -sABORTING_MALLOC=0 \
  -sDEFAULT_LIBRARY_FUNCS_TO_INCLUDE='["$PanicError","$WASM_VECTOR_LEN","$_assertClass","$addOnInit","$cachedTextDecoder","$cachedTextEncoder","$decodeText","$getArrayJsValueFromWasm0","$getArrayU8FromWasm0","$getStringFromWasm0","$passArray8ToWasm0","$passStringToWasm0","$takeFromExternrefTable0"]' \
  -o "$out/cworks-rs.js"

# wasm-bindgen declares its exported classes inside initBindgen while its
# module-scope shims reference them by name; bind the classes to the globals
# declared in pre.js.
sed -i -E 's/^([[:space:]]*)class (FSObj|LuaEnv|Session) \{/\1\2 = class \2 {/' "$out/cworks-rs.js"
