# 作業計画: プロセス毎 _ENV 分離 + stdio/チャネル再編（2026-08・立案済み）

## 前提（ソース確認・ユーザー確定）
- Rust/カーネル**変更なし** → wasm 再ビルド不要。検証は dev server + Playwright（高速イテレーション）。
- **Subscribe は親ディレクトリを作成しない** → チャネルパスの親は spawn 前に存在させる必要あり。
- **stdout チャネル（/run/debug-app/shell-out）は stdio_app.ts が debug_app.ts より先に作成**（シェル spawn 時点で存在）。
- Enter は `\n` 想定（実装時に実測、`\r` なら正規化）。
- 既存の可視化経路は Rust log::error!（env_logger + __ffi_init）と console.error キャプチャ（ConsoleLogger.svelte）のみ。**Lua C print のブラウザ可視性は未確認**。

## 対象ファイル
- 新規: `src/lib/lua/stdio.lua`（内部 `__stdio`）/ `src/lib/lua/bootstrap.lua`（内部 `__bootstrap`）
- 削除: `src/lib/lua/cworks.lua` / `src/lib/lua/cworks-loader.lua`
- 変更: `src/lib/session/luaprocess.ts` / `src/routes/debug/debug/test_proc.lua` / `src/lib/lua/ls.lua` / `src/routes/debug/debug/debug_app.ts` / `src/routes/debug/debug/lua_launcher.ts`
- 維持: `json.lua`（公開）/ stdio_app.ts / textarea_app.ts

## Phase 1: 新規 Lua モジュール
- **stdio.lua**: `build(env,cw) → {print, io, flush}`。遅延書き込み（isyieldable→キュー、flush 契機: 次 print / io.read 待機前 / run 終端 / exit）。sink: env.stdout パス直接 publish（**キャッシュ不要**・シンプル化）、`""`/nil は C print、失敗は C print フォールバック。fake io.stdout/stderr（stderr→stdout）・io.output/io.type 上書き・文字列引数エラー。stdin: subscribe → 文字ストリームバッファ、io.read 全フォーマット（*l/*L/*a/*n/数値/複数/io.lines）、env.stdin=="" はエラー。
- **bootstrap.lua**: 本文で loader_core + グローバル require 上書き + handles（弱キー）レジストリ。loader_core(name, handle_or_nil): キャッシュ→handle 経由 fetch（raw フォールバック）、循環ガード（実行前 package.loaded[name]=true）、Fail→"module not found"。syscall コア: Invoke pcall / until_fn / Fail 素通し。API: list/stat/get/set/mkdir/subscribe/unsubscribe/publish/sleep/exit/wait_for_event。run(env,user_src,name): ハンドル構築→handles[thread]=cw→env.cworks=cw→proc_env={env,print,io,require}→load(...,"t",proc_env)→終端 flush。exit() は _flush()→Done。**ハンドルは thread フィールド持たない**（リーク回避）。

## Phase 2: luaprocess.ts 書き換え
- init: env.run(json) / env.run(stdio) / env.run(bootstrap)
- LuaProcess(name, code, opts: {cmdline, stdout, stdin, extra_args?}) — 必須検証 throw
- 薄いチャンク: `local json = require("json")` → `package.loaded["__bootstrap"].run(json.parse([==[ENV]==]), [==[CODE]==], name)`。**長括弧レベルは code と env JSON 両方走査**、**name も安全埋め込み**（長括弧 or エスケープ）。kernel_callback 現状維持。

## ▷ Gate A（Phase 1+2 後）
- dev server 起動 + 最小プロセス（print→exit）で bootstrap.run / proc_env / require 動作を検証
- **C print 可視性を計測**（print→console キャプチャ）。不可視ならフォールバックは best-effort（現状パリティ、回帰ではない）と文書化

## Phase 3: 既存 Lua 削除・移行
- test_proc.lua: `local cworks = env.cworks`／ローカル stdio 削除→io.read("*l")/io.write／env.cmdline／ta-* 維持／**exec-lua ペイロードに stdout, stdin 追加**（自端末チャネルを渡す）／mkdir("/run","debug-app") は spawn 側へ移管し削除
- ls.lua: `local cworks = env.cworks`、stdio.write → io.write

## ▷ Gate B（Phase 3 後）
- シェル対話・io.read 行入力・ls 起動・**ta-* 回帰（入れ子 syscall の既存挙動を必ず検証）**

## Phase 4: TS 起動フロー更新
- debug_app.ts: cworks fs_set 削除。**spawn 前に fs_mkdir("/run","debug-app")**（Subscribe が親を作らないため）。shell spawn: {cmdline:"sh", stdout:"/run/debug-app/shell-out", stdin:"/run/debug-app/shell-in"}
- lua_launcher.ts: exec-lua の stdout/stdin を受け取り opts 化。**欠落時は warn+skip**（現行バリデーション様式）

## ▷ Gate C（Phase 4 後）
- フルフロー（debug_app→launcher→exec-lua で ls、チャネル受け渡し）検証

## ▷ Gate D（最終 E2E）
- 全シナリオ: 循環 require／__tostring 内 print 遅延と順序／stdout 失敗フォールバック／マルチプロセス隔離／子コルーチン require／stdin 各種フォーマット

## 実装中チェック項目
- fs_mkdir の重複耐性（test_proc と launcher が既に両方 /run を mkdir して共存 → 許容と推測、実測確認）
- 端末 Enter コード実測（\n 想定）
- C print 可視性（Gate A で計測）

## 既知の設計限界（実装時に意識）
- モジュール内ネスト require の子コルーチン初回 raw fetch（狭い窓）
- 入力待ちタイムアウトなし（ハングは仕様）
- *a は「現在バッファ全読み」（EOF 概念なし）

## 実装完了（2026-08-22）
全フェーズ実装・Gates A-D 検証済み:
- Phase 1: stdio.lua / bootstrap.lua 新規（完成）
- Phase 2: luaprocess.ts 書き換え（完成・opts 必須化・薄いチャンクは変数代入形式）
- Phase 3: test_proc.lua / ls.lua 移行（完成）
- Phase 4: debug_app.ts（cworks fs_set 削除・mkdir 追加）・lua_launcher.ts（stdout/stdin 受け取り）完成
- 削除: cworks.lua / cworks-loader.lua
- Gates A-D 全 PASS。C print は console 可視。カーネル特性は design/cworks-runtime-quirks 参照
