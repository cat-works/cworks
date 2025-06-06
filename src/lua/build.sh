#!/bin/bash

script_dir=$(dirname "$0")

# emccの設定
mkdir -p $script_dir/pkg
js=$(realpath $script_dir/pkg/lua-rs.js)
tsd=$(realpath $script_dir/pkg/lua-rs.d.ts)
export EMCC_CFLAGS="-o $js
--emit-tsd $tsd
-s EXPORTED_RUNTIME_METHODS=ccall
-s EXPORT_ES6=1
"

# ビルド
cargo build --target wasm32-unknown-emscripten --release