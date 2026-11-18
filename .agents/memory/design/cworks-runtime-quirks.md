# 実装で判明したランタイム/カーネル特性（2026-08・Gates A-D 検証済み）

プロセス毎 _ENV 分離実装（stdio.lua/bootstrap.lua）の検証で判明した、cworks ランタイムの不可視・重要な特性。

## 検証結果サマリ（Gates A-D 全 PASS）
- Gate A: シェル起動・cmdline 注入・cworks API・exec-lua spawn・**C print はブラウザ console に可視**（emscripten が fd1 を console.log へ配線）
- Gate B: 対話入力・fs コマンド全種・ta-*（textarea push/take、入れ子 syscall）回帰 OK
- Gate D: 循環 require（a.b.a=true）・子コルーチン require・__tostring 内 print の遅延チャンク全件・マルチプロセス隔離・io.read("*n")=42・stdout 失敗→C print フォールバック

## カーネル特性（kernel.rs は変更禁止の前提のため対処は Lua/TS 側）
1. **チャネル配送は FIFO**: 購読プロセスの `outgoing_data_buffer` は `VecDeque::pop_front()`（kernel.rs:191）。同一サブスクライバへの publish は**publish 順に届く**。旧記述の LIFO（`Vec::pop()` / kernel.rs:66）は 2026-09-23 時点で実装と不一致だったことを確認し訂正。→ 生成チャンクで一度に複数 publish する処理も順序が保たれる
2. **アイドルリセット（kernel 終了）**: Running/Sleeping のプロセスが 1 つも無いと kernel.rs:211-213 が `self.processes.clear()` して `start()` のループが抜ける＝**カーネル自体が終了**する（旧実装の `AutoMap::new()` によるプロセス全消去から変更済み）。実アプリは stdio_app が常時 Running（wait_for_event を呼ばない）ため回避。テストハーネスでは keepalive（`p.sleep(0.01)` ループ）が必要
3. **pid 再利用は解消済み**: 旧実装は `AutoMap::new()` によりセッション内で pid を再利用し、FS のチャネル callee_pid リストに残った古い pid が新プロセスに誤配送された。`processes.clear()` 変更により pid カウンタが保持され再利用しない（详见「PID 周辺の変更」節）
4. **Invoke の caller_pid は購読者自身**: kernel.rs:268 は `caller_pid: pid`（publish 元でなく）。Lua ハンドラの caller 引数は信用しないこと

## Lua ランタイムの既知の制約
- **子コルーチンからの syscall は動作しない**: `coroutine.yield` は子コルーチン自身を suspend し、カーネルが resume するのは**メインのプロセスコルーチンのみ**。子コルーチン内の io.write/print/syscall はハング（決して resume されない）。キャッシュ済み require（yield 不要）のみ安全。文書化して受容
- **C print 可視**: `io.write`/`print`（C レベル fd1）は emscripten の console.log に出る。stdout 未設定時のフォールバックは実質可視
- **mlua の生成チャンククセ**: `run(json.parse([=[...]=]), [=[code]=])` の**インライン形式は env が文字列になる**（lua5.2 では正常）。env を一旦ローカル変数に代入してから `json.parse(_envstr)` する形式が堅牢
- **io.read("*n") は確定待ち**: 数値の後にターミネータ（Enter）が来るまでブロック（number_complete が after=="" を不完全扱い）。"12" のみでは待機継続（ハングは仕様）

## 残課題（既存・変更対象外）
- test_proc.lua の `exec` コマンドは**パスでなくファイル内容を exec-lua の path に publish する既存バグ**（launcher が fs_get で失敗）。移行時もそのまま維持した（計画対象外）

## TS Process の wait_for_event トークンレース（2026-08・修正済み）
- **症状**: shell で `exec /usr/bin/ls.lua` しても ls が起動しない（起動時の exec は動く）
- **原因**: launcher の `while(1) { await p.wait_for_event(); }` パターン。`wait_for_event()` は result_queue に `"WaitForEvent"` トークンを積む。Invoke 受信時、subscribe ハンドラが積んだ `fs_get` の Get リクエストが**古い WaitForEvent トークンの後ろ**に回り、`kernel_callback` の shift() がトークンを返して process が睡眠 → Get が永遠に処理されない（起動時 exec はトークンが先に消費されるタイミングで動く）
- **修正**: launcher / textarea_app のループを `while(1) { await p.pending(); }` に変更（`wait_for_event` をやめ常時 Running、stdio_app と同パターン）。`pending()` はトークンを積まないため、ハンドラのキュー積みが即座に shift される
- **留意**: `wait_for_event()` + subscribe ハンドラで result_queue に仕事を積むパターンは同レースを持つ。TS Process で Invoke をトリガーに fs_* を呼ぶアプリは pending() ループを使うこと

## TS Process の wait_for_event トークンレース（2026-08・修正済み・v2）
- **症状**: shell で `exec /usr/bin/ls.lua` しても ls が起動しない（起動時の exec は動く）
- **原因**: `wait_for_event()` は result_queue に `"WaitForEvent"` トークンを積む。イベント後ループが再積みしたトークンが**休眠中に消費されず残り**、次の Invoke 受信時、subscribe ハンドラが積んだ `fs_get` の Get が**古いトークンの後ろ**に回って shift() がトークンを返し、プロセスが再睡眠 → Get が永遠に処理されない
- **修正（v1→v2）**:
  - v1（非推奨）: launcher/textarea を `while(1){ await p.pending(); }` に変更（常時 Running = busy-poll、CPU 非効率）
  - v2（採用）: **`Process.wait_for_event`（process.ts）を修正** — once ハンドラが実イベント（x ≠ "None"）で起きた時、result_queue から `"WaitForEvent"` トークンを除去してから resolve。これでハンドラが積んだ仕事が shift で即座に返り、**プロセスは休眠のまま**（WaitForEvent 返却）正しく処理できる
- **確認**: exec 動作 + launcher が WaitForEvent で休眠復帰（Get 即送信）。Gate A/B 全 PASS
- **教訓**: TS Process で「Invoke を受けてハンドラが fs_* を積む」パターンは、`wait_for_event` のトークン除去を Process 側で行うのが正解。pending() ループ化は busy-poll になるため避ける

## 統合 emscripten モジュール移行時の JSON 境界問題（2026-08・解決済み）
- **背景**: src/wasm (wasm-bindgen kernel) + src/lua/pkg/lua.ts を src/lua/pkg/cworks.ts (emscripten 統合モジュール) に統一した。旧パスは wasm-bindgen が JS オブジェクト/BigInt をネイティブ変換していたが、新パスは Rust↔JS 間がすべて JSON 文字列経由。
- **症状1**: `children_map.get is not a function` — wasm-bindgen は HashMap を JS Map で渡すが、JSON 経由ではプレーンオブジェクトになる。`lua_launcher.ts` を `.get(k)` → `[k]` アクセスに修正。
- **症状2**: シェルが `wait_for_process` 後に Done で死亡しプロンプト復帰しない — `PollResult::WaitForProcess(u128)` のペイロード (`$$bi:3` → BigInt) を JSON.stringify すると BigInt は文字列 `"3"` になり serde が u128 をデシリアライズできず `ffi_session.rs` の `unwrap_or(PollResult::Done)` で即死。**修正: replacer で bigint → Number(value)**（PID 程度の小さい値なら精度損失なし）。`value.toString()` は不可（文字列化で serde 失敗）。
- **教訓**: ffi_session.rs の `unwrap_or(Done)` はデシリアライズ失敗を静かに握り潰す。JS 側から返す PollResult JSON の型は serde の期待に正確に合わせる必要がある（u128 = JSON 数値）。
- **検証方法**: Playwright で xterm-helper-textarea に focus → keyboard.type("ls", delay=100) → Enter。**ls を 2 回実行してプロンプト復帰を確認するのが回帰テスト**（wait_for_process の死が検出できる）。

## PID 周辺の変更（2026-08・GetPid 実装に伴う更新）
- **`PollResult::GetPid` / `SyscallData::GetPid(u128)` 追加**: プロセスは `cw.get_pid()` で自身の pid を取得可能（「自己識別不能」制約は解消）。bootstrap の simple() が GetPid 応答をアンラップ済み
- **pid は 1 開始**（0 はカーネル予約）。createVFS 設計の「PID=0 ⇒ Kernel」と整合
- **アイドルリセットが `processes.clear()` に変更**（旧: `AutoMap::new()` で作り直し）→ pid カウンタが保持され、**セッション内での pid 再利用は発生しなくなった**。旧特性 #3「pid 再利用によるチャネル誤配送」は解消（callee_pid 残留問題は今後は死亡 pid 参照=NoSuchEntry 系に変化）
- test-cli は簡素化（cworks-lua/mlua/serde_json 依存除去、純 Rust プロセスの playground に回帰）

## wasm-bindgen 移行後のランタイム特性（2026-09）

- **JS Process が waitForEvent で休眠中に外部起点の要求は送れない**: 現行 `Process`（src/lib/session/process.ts）は `request()` でキューに積むだけで、送信（pump）は `kernelCallback` の**非 invoke 経路**でのみ実行される。`emitter.send`（DOM イベント起点）のようにカーネルコールバック外から `p.publish` すると、プロセスが WaitingForEvent のままだと次の poll が来ず永久に送信されない。→ 外部起点で publish するサーバプロセスは**常時 Running** を保つ（`while(1) await p.waitForEvent()` ではなく `await new Promise(() => {})`）。stdio_app が機能するのは常時 Running（waitForEvent を呼ばない）ため。
- **wasm-bindgen/emscripten の HEAP アクセサ shim**: 生成 JS が `HEAP320()` / `HEAPU80()` / `HEAP_DATA_VIEW0()` を呼ぶが未定義。`wasm/cworks-wasm/pre.js` に `HEAP320()->HEAP32`, `HEAPU80()->HEAPU8`, `HEAP_DATA_VIEW0()->new DataView(HEAPU8.buffer)`（buffer 変化時に再生成）を定義する。`FSObj.list()`（`Vec<String>` 返却）で `getArrayJsValueFromWasm0` 経由で顕在化した。
- **FSObj ツリーの親は Weak 化済み**: `Object::CompoundFSObj.parent` は `Weak<RefCell<Box<Object>>>`（Rc サイクル解消）。`FSObjRef::remove_child(name)` を新設（親 children から除去＋子 parent=None、`NoSuchEntry`）。Lua `remove_child` / JS `removeChild` に公開。外部 crate から `Object::CompoundFSObj` を直接構築する場合は `FSObjRef::new_compound(parent)` を使う（`Rc::downgrade` は crate 外から不可）。
