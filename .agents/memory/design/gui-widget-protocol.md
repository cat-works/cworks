# GUI / ウィンドウマネージャ設計（2026-09・案A・WM 分離）

## 全体像
- **Display server (DS)** = `src/routes/debug/debug/gui_app.ts`（JS プロセス）。`/debug/debug` の xterm 上に `<Desktop>` を重ねる。
- **Window manager (WM)** = `src/lib/lua/usr/bin/wm.lua`（Lua プロセス、既定 WM）。`debug_app.ts` が exec-lua で自動起動。
- **app** = `gui.lua` で window を要求し、返ってきた **Grid** を直接操作する。

## オブジェクトモデル（id レス）
```
Display (DS 生成/所有, kind="display", control/event 付き)
 └─ frame (WM 構築のウィジェットツリー。根に x,y,w,h,z, wm_name props)
     ├─ decoration ウィジェット (titlebar=grid[grip(label,drag=move), close(button)], footer=grid[spacer, resize(label,drag=resize)])
     └─ body (Grid)  ← WM が生成し app に返す content 面
```
- ウィンドウ状態は frame FSObj 上の props。**WM は台帳を持たず Display ツリーが台帳**（stacking/配置は Display の子を走査）。
- メッセージは **FSObj ハンドル**を運ぶ（`window=<frame>`）。数値 id なし。WM は渡された FSObj を直接 mutate。
- Display の直子はすべて frame として解釈（taskbar 等も frame）。DS は Display ツリーを汎用描画（`x,y,w,h,z` で絶対配置、`rows` prop 付き grid は height:100%）。
- frame の `wm_name` prop は Display 内の子名（`remove_child` に必要）。
- ウィンドウ操作は **Grid の `control`**、イベントは **Grid の `event`**。

## チャネル / プロトコル（CompoundFSObj）
- `/run/gui/req`（app→DS）: `{op="create", title, options, x,y,w,h, reply_to:Func}`（DS はプロキシ）
- `/run/gui/wm-req`（DS→WM）: `attach{display}`, `create{reply_to,...}`, `focus{window}`, `move{window,x,y}`, `resize{window,w,h}`
- `/run/gui/wm-res`（WM→DS）: `sync`（WM 起動時に publish。DS は `attach` を再送）, `ready`（DS は保留 create を flush）
- WM→app: Grid.event に `{event="focus"|"move"|"resize"|"close", x,y,w,h}`
- app→WM: Grid.control に `{op="close"}`（WM が購読）
- 変更通知: WM は Display/frame を mutate し Display.control に `{op="changed"}`。DS は Display を再走査（全 control 購読、subscribe 後 recheck）。

## Lua API（`gui.lua`）
- `local grid = gui.request{ title, options, x,y,w,h }`（reply 待ち）→ Grid を操作。
- `grid:label/button/input/html/grid{id, area, ...}`、`grid:on(event, fn)`、`grid:get(id)`、`grid:set(key,v)`、`grid:close()`。
- **重要**: `ProcessSession` は 1 Func につき 1 ハンドラ（`data_handlers` を as_ptr でキー）。よって `:on` は widget ごとに **1 回だけ subscribe し、内部でイベント名→複数ハンドラに多重化**する。wrapper は `children` でキャッシュし、同一 widget には同一 wrapper を返す（再 subscribe で上書きされないため）。
- `api.widget(kind, opts)` は窓登録を伴わないスタンドアロンツリー（WM の frame 構築用）。
- ウィジェット prop: `area="r,c-r2,c2"`（0始まり半開、CSS grid-area へ +1）、`drag="move"|"resize"`（DS が pointer 捕捉）、`columns`/`rows` は数値 or 文字列テンプレート。

## DS の要点
- **常時 Running**（`await new Promise(() => {})`）。`waitForEvent` で休眠すると DOM 起点の `emitter.*` publish が pump されない。
- `emitter.send(widgetId, event, value)` は widget.event へ、`emitter.focus/move/resize(window, ...)` は wm-req へ。
- `Window.svelte` が frame ツリーを描画し、`on:pointerdown` で focus 送信＋`[data-drag]` から WM 駆動 drag/resize（rAF スロットル、ローカル移動なし）。
- `Widget.svelte` の grid item wrapper に `data-widget-id`/`data-drag` を付与。HashMap 順序に依存しないよう **frame 内は area で明示配置**。

## 落とし穴
- Display の子は HashMap 順 → frame 内の titlebar/body/footer は必ず `area` で配置。
- `require` したモジュール env の `print` は C fd1（端末に出ない）。
- close ボタン click は window の pointerdown も発火し focus も送られる（正常）。
- 本設計は旧 Window ウィジェット実装の作り直し（破壊的）。

## E2E（Playwright, chromium-cdp）
`pnpm run dev` → `/debug/debug` → ターミナルに `gui-demo` → ウィンドウ表示 → input+Greet でラベル更新 → titlebar drag で移動 → ◢ でリサイズ → × で消滅 → 2窓で focus z-order → console error 0。ターミナル入力は `page.keyboard.type`。

## モバイル対応と背景（2026-09）

- **タッチのドラッグ荒ぶり対策**: ドラッグハンドル（`[data-drag]`）に `touch-action:none;user-select:none;` を付与（`Widget.svelte` の grid item wrapper）。`Window.svelte` は `setPointerCapture` を使い、`pointermove`/`pointerup`/`pointercancel` をハンドル上で受ける（`pointerId` を照合）。これでブラウザのスクロール/ジェスチャによる `pointercancel` と瞬間移動を防止。
- **フォーカス**: 入力要素はキー付き each により DOM が保持され、タップ後も `document.activeElement` は維持される（touch エミュレーションで確認）。ドラッグハンドルのみ `preventDefault` し、入力タップは阻害しない。
- **背景の仕組み**: `Display` の `background` prop（任意の CSS `background` 値）を DS が読み、`Desktop.svelte` のオーバーレイに適用（`state.ts` の `background` store）。既定は WM が `attach` 時に設定（半透明グラデーション＝ターミナルを完全に隠さない）。変更は Display の `background` を書き換えて `changed` を送れば反映（WM が所有者）。
- DS の reconcile は Display の子を **`type()==="CompoundFSObj"` かつ `kind` を持つものだけ** frame として扱う（`background` 等のプロパティ子を誤読しない）。
- 検証: CDP `Emulation.setTouchEmulationEnabled` + `Input.dispatchTouchEvent` でタッチのドラッグ/リサイズ/close/入力フォーカスを確認、console error 0。
