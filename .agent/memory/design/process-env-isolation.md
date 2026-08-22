# プロセス毎 _ENV 分離 + stdio/チャネル設計（2026-08 最終版・確定・2 回吟味済み）

## アーキテクチャ
- 単一 lua_State。プロセス毎に `_ENV`（proc_env）を与える。lua_State 多重化はチャネル/`Object::Func` が mlua 状態に束縛されるため不採用。
- **モジュールは env 非依存**（`load(content)` で _G コンパイル）。共有 `package.loaded`。プロセス固有 I/O は env 袋・ハンドル・引数で明示注入。
- カーネルは **Subscribe 時にチャネルを自動生成**する（確認済み）。env のパスは pre-create 不要。

## ファイル構成（再編後）
- `json.lua` — 公開（現状どおり preload）
- `stdio.lua` — 内部 `__stdio`（ビルダー `make_print(env,cw)` / `make_io(env,cw)`）
- `bootstrap.lua` — 内部 `__bootstrap`（**旧 cworks.lua + cworks-loader.lua + bootstrap を統合**。cworks モジュールは廃止）
- Rust / カーネルは変更なし

## bootstrap.lua
- 初期化: グローバル require 上書きをインストール → `loader_core(name, handles[coroutine.running()])`（弱キー thread→handle レジストリ参照）
- `run(env, user_src, name)`:
  1. syscall コア + API ハンドル構築（list/stat/get/set/mkdir/subscribe/unsubscribe/publish/sleep/exit/wait_for_event。ハンドラ表はハンドル内 = プロセス内 1 本）
  2. `handles[coroutine.running()] = cw`（弱キー）
  3. `env.cworks = cw`
  4. proc_env = setmetatable({ env, print, io, require=loader_core クロージャ(cw) }, { __index = _G })
  5. `load(user_src, "@"..name, "t", proc_env)` → 実行 → 終端 flush
- **ハンドルは thread フィールドを持たない**（弱キーレジストリ value→key 強参照によるリーク回避）
- **`cw.exit()` は flush_pending() してから `syscall("Done")`**（スレッド死亡で終端 flush に到達しないため）

## require / loader_core(name, handle_or_nil)
- キャッシュ（package.loaded / package.preload）→ fetch は **handle の syscall 経由**（Invoke ディスパッチ対応、生 yield 廃止）
- **循環 require ガード**: `package.loaded[modname] = true` を chunk 実行**前**に立てる
- グローバル require（共有）: レジストリ参照。proc_env.require（クロージャ）: cw 直渡し → 子コルーチンでも字義的 _ENV 継承でハンドル経由
- 既知のエッジ: モジュール内ネスト require が子コルーチン初回実行 → レジストリ miss → raw fetch フォールバック（fetch は機能。Invoke 同時着弾の狭い窓のみ。文書化して受容）

## syscall コア
- Invoke 受信 → handle.handlers[path] を pcall で実行 → until_fn が真なら返る／なければ "Pending" で再帰
- **Fail 応答は素通し**（呼び出し側が解釈）
- ハンドラ内からの再入 syscall は安全（ループ状態は per-call ローカル）— 仕様として文書化

## env 袋（spawn 側が注入、bootstrap が runtime 追加）
`{ cmdline: string, stdout: string, stdin: string, extra_args?: any, cworks: <handle> }`
- チャネル解決はここ（パス文字列渡し）。mlua Function 直渡しは不採用。`env.name` は含めない。
- **`env.stdout == ""` は未設定とみなし C print 委譲**（オプトアウト可）
- **`env.stdin == ""` はエラー**（入力のオプトアウトは不可。実チャネルパス必須）

## stdio（遅延書き込み）
- sink: env.stdout パス → 初回 emit 時に handle.publish({String=chunk}) として解決しキャッシュ。未指定/`""` は C レベル print 委譲（ゼロ回帰）。
- **stdout publish 失敗は C print へフォールバックして継続**（best-effort）。
- stderr → stdout へリダイレクト。
- 非 yield 文脈（isyieldable() false）: キューに積む。flush 契機: ① 次の print/io.write（先に flush で順序保証）② bootstrap 終端 ③ io.read の待機開始前 ④ cw.exit()。
- flush 失敗も best-effort（C print 可視化）。バッファ無制限。
- fake io.stdout/stderr: write / flush / close(noop) / setvbuf(noop)。io.output と io.type は上書き必須。**io.output/io.input の文字列引数（ファイルパス）は明示エラー**（fake/省略のみ対応）。

## stdin（入力）
- make_io が env.stdin を subscribe → ハンドラは `{String=chunk}`（キーストローク断片）を**文字ストリームバッファ**に追記。`"\r"` は `"\n"` に正規化するか実装時に確認。
- **io.read は Lua 標準フォーマットを網羅**: `*l` / `*L` / `*a` / `*n` / 数値 n / 複数フォーマット同時指定 / `io.lines()`。待機述語はフォーマット別。
- 非 yield 文脈でバッファ不足 → 明示エラー。
- **入力待ちにタイムアウト機構はない（ハングは仕様）**。待機中も他チャネルの Invoke はディスパッチされる（応答性維持）。
- warn() 対象外。

## 生成チャンク（TS）
- init: `env.run(json)` / `env.run(stdio)` / `env.run(bootstrap)`
- spawn: 薄いチャンク。**`local json = require("json")` を使ってから** `package.loaded["__bootstrap"].run(json.parse([==[ env JSON ]==]), [==[ user code ]==], "name")`（json はグローバルではないため）。
- **長括弧レベルの走査対象はユーザーコードと env JSON の両方**（出現しない level を選択）。preamble 廃止。

## LuaProcess（TS）
```ts
new LuaProcess(name, code, opts: { cmdline: string; stdout: string; stdin: string; extra_args?: any })
```
- cmdline/stdout/stdin は**必須**（未指定 throw）。stdout 不要プロセスは `""` を渡す（stdin は `""` 不可）。extra_args 任意（既定 {}）。

## 仕様変更明細（確定）
1. luaprocess.ts: preamble 廃止、init 3 モジュール、LuaProcess opts 必須化
2. cworks.lua: **削除**（bootstrap.lua へ統合）
3. cworks-loader.lua: **削除**（bootstrap.lua へ統合）
4. 新規 bootstrap.lua / stdio.lua
5. test_proc.lua: `local cworks = env.cworks;` + ローカル stdio → print / io.read / io.write + get_cmdline() → env.cmdline
6. ls.lua: 同上（ローカル stdio.write → io.write）
7. debug_app.ts: LuaProcess 起動時に stdout/stdin パスを opts で渡す
8. Rust: 変更なし

## 既知の限界
- モジュール内ネスト require の子コルーチン初回 raw fetch（狭い窓）
- ユーザーコードの素の load() は _G に落ちる（セキュリティ境界ではない）
- バッファ無制限、warn 対象外、`*a` は EOF 概念がないため「現在のバッファを全読み」
- 入力待ちタイムアウトなし（ハングは仕様）
- ハンドル内ハンドラ表はプロセス内 1 本（channel_handlers 共有衝突は解消済み）
