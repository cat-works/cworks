# wasm ツールチェーンのバージョン制約（emscripten × wasm-bindgen）

## TL;DR
cworks の wasm ビルド（`wasm/cworks-wasm/build.sh`）は emscripten 6.0.1+ の実験的 `-sWASM_BINDGEN` 統合に依存する。emcc が内部で `wasm-bindgen` CLI を PATH から呼ぶため、**crate の wasm-bindgen と CLI はマイナーバージョンまで厳密一致が必須**。ツールチェーンは unstable チャンネル（flake.lock で rev 固定）で運用する。

## バージョン矩阵（2026-11 時点の動作構成）
- nixos-unstable (rev ピン) + emscripten **6.0.2** + wasm-bindgen-cli **0.2.121** + crate `wasm-bindgen = "=0.2.121"` → 全部一致でビルド成功、自前ビルド不要
- nixpkgs 標準 wasm-bindgen-cli（unstable・26.05 とも 0.2.121）に crate を合わせるのが最小構成

## 経緯と判断
- emscripten の wasm-bindgen 統合（`-sWASM_BINDGEN`）は **6.0.1 (2026-06) で追加 (#23493)、6.0.9 で experimental 扱い (#27616)**。nixos-26.05 の emscripten 5.0.6 (2026-04) には存在せず、`Attempt to set a non-existent setting: 'WASM_BINDGEN'` で失敗する。emscripten 6.x が stable チャンネルに入るのは nixos-27.05 以降の見込み
- 26.05 移行を試みたが上記により不可。**flake.lock の rev ピンが再現性の本質**であり、channel の stable/unstable は本質でない、と判断して unstable 復帰（2026-11 確定）

## CLI を crate に合わせる場合の nixpkgs 標準手段
- `pkgs.buildWasmBindgenCli` build-support（26.05/unstable に存在）で任意バージョンの CLI をソースビルドできる（src hash は `fetchCrate` の error loop、cargoDeps hash は `fetchCargoVendor` の error loop で確定）。ただし crate を 0.2.121 に合わせる限り不要
- バージョン付き attr（`wasm-bindgen-cli_0_2_*`）も同じ目的で nixpkgs が提供

## 落とし穴
- `-sWASM_BINDGEN` 統合は実験的で「features may not yet be fully supported」（6.0.1 ChangeLog）。emscripten マイナーアップデートで挙動が変わりうる
- crate 側を `=0.2.130` 等に上げる場合は CLI も一致させる必要があり、nixpkgs 標準に存在しないバージョンなら buildWasmBindgenCli での自前ビルドが発生する
- `wasm-bindgen-futures` はバージョンごとに wasm-bindgen に厳密ピンされる（0.4.80 → `=0.2.130`）。未使用なら依存から外すのがバージョン更新の障害を減らす（cworks では 2026-11 に未使用のため削除）
