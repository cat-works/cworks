#!/bin/bash
set -e

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

# pkgディレクトリ作成
mkdir -p "$script_dir/pkg"

# mluaのvendored Luaをemscripten向けにクロスコンパイルする。
# TARGET_CC=emcc は flake 側 (devShell) が設定する。
# -fwasm-exceptions: Rust 側(rustc)と同じ wasm 例外モードに揃える
#   (外すと __cxa_find_matching_catch_3 が未定義でリンクに失敗する)
export TARGET_CFLAGS="-fwasm-exceptions"
export TARGET_CXXFLAGS="-fwasm-exceptions"

# JS グルー生成用のリンク引数は build.rs が設定する
# (cargo:rustc-link-arg=-o<lua-rs.js> など)

# ビルド
cargo build --target wasm32-unknown-emscripten --release
