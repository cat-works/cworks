# 実装で判明したランタイム/カーネル特性（2026-08・Gates A-D 検証済み）

プロセス毎 _ENV 分離実装（stdio.lua/bootstrap.lua）の検証で判明した、cworks ランタイムの不可視・重要な特性。

## 検証結果サマリ（Gates A-D 全 PASS）
- Gate A: シェル起動・cmdline 注入・cworks API・exec-lua spawn・**C print はブラウザ console に可視**（emscripten が fd1 を console.log へ配線）
- Gate B: 対話入力・fs コマンド全種・ta-*（textarea push/take、入れ子 syscall）回帰 OK
- Gate D: 循環 require（a.b.a=true）・子コルーチン require・__tostring 内 print の遅延チャンク全件・マルチプロセス隔離・io.read("*n")=42・stdout 失敗→C print フォールバック

## カーネル特性（kernel.rs は変更禁止の前提のため対処は Lua/TS 側）
1. **チャネル配送は LIFO**: 購読プロセスの `outgoing_data_buffer` は `Vec::pop()`（kernel.rs:66）。同一サブスクライバへの**急速な連続 publish は逆順で届く**。実シェルでは syscall 間隔で publish が離れるため問題化しない。→ 生成チャンクで一度に複数 publish する処理は順序が逆転しうる（仕様として許容）
2. **アイドルリセット**: 全プロセスが WaitingForEvent/Sleeping でないと kernel.rs:321-323 が `AutoMap::new()` で**全プロセスを消去**。実アプリは stdio_app が常時 Running（wait_for_event を呼ばない）ため回避。テストハーネスでは keepalive（`p.sleep(0.01)` ループ）が必要
3. **pid 再利用**: AutoMap が空き pid を再利用。FS のチャネル callee_pid リストに**古い pid が残ると、再利用後の新プロセスに誤配送**される
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
