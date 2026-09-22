# GUI ウィジェットプロトコル（2026-09・P0 実装済み）

## 方針
- `/debug/debug` の xterm 上に `<Desktop>` オーバーレイを重ねる（ターミナルは維持）。
- ウィジェットは FSObj（プロセス非紐付けの値）。**Window もウィジェット**。
- ウィジェットごとに `control`（変更シグナル）と `event`（ユーザーイベント）の Func チャネル（per-widget）。
- **FSObj ツリーが唯一の真実**。`control` は `{op="changed"}` / `{op="close", node=...}` のみを送り、サーバがノードを再読込（冪等・dedupe 不要）。将来カーネル FSObj 監視（userspace channel の後）を入れる際は手動シグナルを捨てるだけで移行できる watch-ready 設計。
- ウィジェット種: Window, Grid, Label, Button, Input(`type`=text/password/number), HTML(raw)。canvas/list/tree/checkbox/select は合成で対応（プロトコル外）。
- メッセージは CompoundFSObj。Grid 配置は `area = "<row>,<col>-<row_end>,<col_end>"`（0始まり・半開、描画時に CSS grid-area へ +1 変換）。

## 構成
- ウィジェット FSObj の子: `kind`(String), `control`(Func), `event`(Func), `content`(CompoundFSObj), プロパティ。
- Window の `content` は 1 子のみ（既定 Grid, columns=1）。`win:get_grid()` で取得。
- 制御チャネル `/run/gui/req` に Window FSObj を publish して登録。サーバが content 以下を再帰走査して購読・描画。
- 競合対策: (1) 登録時に live ツリーを再帰走査して初期子を拾う、(2) 新規 control 購読確立後に一度 reconcile し直して購読前の変更を拾う、(3) 追加/設定は常にツリーを mutate し `changed` を publish（再読込は冪等）。

## Lua API（src/lib/lua/usr/lib/gui.lua）
`require("gui").new(session, env)` → `gui.window{title,options,x,y,w,h}`（self なし）→ `win:get_grid()` → `grid:label/button/input/html/grid{id,area,...}`、`widget:on(event,fn)`、`widget:set_text/set_value/set_html/set_title/set_options(v)`、`widget:remove()`、`win:close()`、`grid:get(id)`、`widget:prop/value()`。
- `:on` のコールバックはイベント購読ハンドラ内で**そのまま syscall する**（`set_*`/publish）。`ProcessSession::do_syscall` が invoke を内部でハンドラにディスパッチし、ネスト syscall も応答スロット共有で動く。
- 注意: require したモジュール env の `print` は `_G`（C レベル fd1）に行き、proc_env の `st.print`（端末）には出ない。モジュール内デバッグ print は端末に表示されない。

## 実装ファイル
- `src/routes/debug/debug/gui_app.ts`（JS サーバ。常時 Running で待機＝外部起点 publish を pump させる）
- `src/lib/desktop/state.ts`（Svelte `writable` store + `emitter.send`）
- `src/lib/components/Desktop.svelte`, `src/lib/components/desktop/{Window,Widget}.svelte`（Card/Button 再利用）
- `src/lib/lua/usr/lib/gui.lua`, `src/lib/lua/usr/bin/gui-demo.lua`
- `src/routes/debug/debug/+page.svelte` に `<Desktop>` と gui プロセス起動を追加

## 未実装（P1/P2）
- ウィンドウのドラッグ/リサイズ/最小化/最大化/スナップ。現状はクリックで前面（簡易 z-order）のみ。
- `set_content`、`resize` イベント。HTML は `{@html}` で XSS 許容（協調前提）。

## E2E（Playwright, chromium-cdp）
`nix develop --command pnpm run dev` → `/debug/debug` → ターミナルに `gui-demo` → ウィンドウ表示 → Input 入力 + Greet → ラベル更新 → Close で消滅、console error 0。
- ターミナル入力は xterm のため `page.keyboard.type` を使う（`fill` は不可）。`+page.svelte` の `onData` をバッファリング化して高速入力の取りこぼしを解消。
