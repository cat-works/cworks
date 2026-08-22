# リファクタ仕様: process-env 実装の整理（2026-08・確定）

design/process-env-isolation 実装（git diff 内）のリファクタ。**挙動完全維持**（Gates A-D 検証済みの現挙動を保持）。ユーザー選択: 構造改善 B + 重複排除 A、対象は `src/lib/lua/*.lua` + `src/lib/session/luaprocess.ts` のみ（test_proc.lua / lua_launcher.ts / debug_app.ts / ls.lua は対象外）。

## 1. luaprocess.ts — 薄いチャンクを Lua テーブルリテラルに
- **`toLuaLiteral(v)`** 再帰シリアライザ: string→長括弧 / number→`String(v)`（非有限は nil）/ boolean→true|false / array→`{ v, ... }` / object→`{ key = v, ... }`（キーは `luaKey(k)`: 識別子なら裸・他は長括弧）/ null|undefined→nil / その他→throw
- **`longBracket(s)`**: 現 `maxBracketLevel` の可変長引数を単一文字列版に簡約（実使用は単一のみ）
- **`buildThinChunk(env, code, name)`**:
  ```lua
  local env = <toLuaLiteral(env)>
  package.loaded["__bootstrap"].run(env, <longBracket(code)>, <longBracket(name)>)
  ```
  `require("json")` / JSON.parse は消滅（mlua のインライン parse クセも回避）
- コンストラクタ: env bag は **`procEnv`** と命名（モジュールの LuaEnv `env` をシャドウしない）。opts 必須・stdin≠"" 検証は維持。kernel_callback 変更なし

## 2. stdio.lua
- **`deliver(chunk)`** 抽出: publish→非 FSSuccess なら C write フォールバック（emit_out/flush_pending の重複解消）
- **`read_handlers` 表駆動** + `normalize_read_format`（先頭 `*` 除去・number→count・string/number 以外はそのまま→既存エラー文言維持）:
  - a: drop=#buf, val=buf / l: find \n→drop=nl, val=sub(1,nl-1) / L: drop=nl, val=sub(1,nl) / n: number_complete→drop=consumed, val=num / count: #buf>=n→drop=n, val=sub(1,n)
  - read_one: `(drop, val)` or nil → `in_buf = in_buf:sub(drop+1)`
- **`make_handle(write_fn, flush_fn)`** ファクトリ: stdout/stderr は write/flush 共有（stderr→stdout 維持）、stdin は write/flush なし
- 許容微差: `io.stderr:write` 戻り値が stdout_handle でなく自身になる（Lua 標準へ是正、影響なし）

## 3. bootstrap.lua
- **`ALWAYS = function() return true end`**（exit/run終端/simple 共用）
- **`parse_fetch_response(res, modname)`** + `fetch_done` 述語で cw 経由/raw フォールバックの応答検証を共通化
- **`build_handle()`**: 未使用 env 引数削除・メソッド群をテーブルリテラル化
- **`run()`**: process_flush upvalue 舞い廃止 → `cw.exit = function() st.flush(); cw.syscall("Done", ALWAYS) end`（st はスコープ内）

## 検証（Build エージェント必須）
1. lua5.2 で `/tmp/opencode/cworks-e2e/test_stdio.lua` 再実行 → 全 PASS
2. dev server + Playwright Gate A（**薄いチャンクの mlua 動作を必ず確認**）→ Gate B → Gate D
3. svelte-check 変更ファイルにエラーなし
4. 既知挙動（LIFO 配送・子コルーチン制約・アイドルリセット）が変化しないこと

## リスク
- mlua が関数呼び出し引数内テーブルリテラルでクセを踏む可能性（過去に json.parse インライン引数で env が string 化）→ **変数代入形式 `local env = {...}` を既定**、Gate A で確認
- toLuaLiteral の null→nil でキー脱落（extra_args 内 null は消える。現状使用なし）

## 引き継ぎ先
Plan / Build エージェントで上記 3 ファイルを修正し、検証手順を実施する

## 実装完了（2026-08-22）
- luaprocess.ts / stdio.lua / bootstrap.lua の 3 ファイルを本仕様どおりリファクタ
- 検証: lua5.2 ユニット ALL PASS / Gate A・B・D 全 PASS / svelte-check 変更ファイルエラーなし
- テーブルリテラルの薄いチャンクは mlua で正常動作（変数代入形式 `local env = {...}` を採用し問題なし）
