# wasm クレートの役割設計（2026-08・Native LuaProcess アーキテクチャ）

**ユーザーが成果物フリップ後さらに再構成（コミット f911aa5〜1afc288）。本書は現行アーキテクチャの記録。**

## 現行アーキテクチャ

| ディレクトリ | パッケージ名 | 役割 |
|--------------|--------------|------|
| `wasm/wasm` | **cworks** (cdylib) | JS 境界の一切: `ffi_session`(SESSION pub)/`ffi_luaprocess`/`ffi_luaenv`/`js_callback`/`__ffi_init`/`__ffi_demangle` + glue + build.rs + pkg |
| `wasm/lua` | **cworks-lua** (rlib, deps mlua+kernel+serde_json+log) | `LuaProcess`: kernel::Process を実装するネイティブ Lua プロセス |
| `wasm/kernel`, `wasm/test-cli` | 変更なし | |

## 核心: Native LuaProcess（syscall ループの Rust 完結化）

```
Lua プロセス:  Kernel ⇄ cworks_lua::LuaProcess::poll ⇄ thread.resume(JSON文字列) ⇄ Lua
JS プロセス:   Kernel ⇄ JsCallbackProcess ⇄ cworks_js_callback ⇄ TS Session コールバック
```

- `LuaProcess::poll`: SyscallData を serde_json で文字列化 → resume → 戻り値文字列を PollResult としてデシリアライズ
- **buf_encoding 削除**: Rust↔Lua 間は C 文字列境界を通らないため NUL エスケープ不要に（JSON に生 NUL は出現しない）。TS 側 encode/decode も消滅
- **LuaEnv/LuaThread 型付き API・FFI 廃止**: `__ffi_lufenv_new` は生 `*mut mlua::Lua` を返す。wasm crate は mlua を直接依存（「mlua 型非漏出」ルールは撤回）
- `__ffi_lua_process_new(env, name, code) -> pid`: コードを関数化→create_thread→LuaProcess で SESSION に登録
- **CLI 対応が実質完了**: 同一 `cworks_lua::LuaProcess` が native (test-cli で実証済み) / wasm の双方で動作

## TS 側の変化

- `luaprocess.ts`: kernel_callback/yield ループ廃止。薄いチャンク生成＋`createNativeLuaProcess()` 呼ぶだけ
- `cworks.ts`: Session(JS プロセス用)/demangle_str/createNativeLuaProcess/LuaEnv(run のみ) に縮小
- bootstrap.lua: `WaitForProcess = pid` 直接数値化（旧 `$$bi:` プレフィクス廃止 — JSON が直接 serde されるため）

## 検証済み（2026-08・コミット済み状態）

cargo check --workspace ✓ / test-cli ネイティブ LuaProcess 動作 ✓ / ブラウザ回帰 ls×2 プロンプト復帰 ✓ ページエラー 0 ✓

## 将来構想: CLI フロントエンド

Native LuaProcess 化により技術的基盤はほぼ整備済み。残る差分は stdio 端点(TTY)とパッケージングのみ。
`.lua` パーソナリティ資産 (`src/lib/lua/*.lua`) のワークスペース共有化は今後の作業（先送りのまま）。

## 関連メモリ
- `design/cargo-workspace-layout` — レイアウト・命名
- `design/cworks-runtime-quirks` — カーネル特性（JSON 境界問題の記録あり。$$bi 関連は一部過去のものに）
