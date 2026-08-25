# Kernel-Process Dialogue / 仮想オブジェクト設計（2026-08・最終確定）

## 構成
- **`Object::Virt { req_ch, rep_ch }`** — チャネルパス 2 本を保持するノード
- **`CreateVFS`** — Virt 設置＋req_ch/rep_ch チャネル生成＋**rep_ch の callee_pid に 0(Kernel) を追加**

## プロトコル
```
Req ≡ { requested_by: Pid, method: Stat|List|Get|Set|Mkdir|Pub, path, content }
Res ≡ { reply_to: Pid, method, content }   // method をエコー（カーネル ステートレス）
```

## フロー
```
1. App(X) syscall → 解決先が Virt → Req を req_ch へ配送(caller_pid=0)、X は WaitingForEvent
2. Provider が Req 受信（requested_by で要求元把握）→ 通常 syscall で処理
3. Provider が Res を rep_ch へ publish → callee_pid[0] でカーネル横取り
4. method で型合成（Get→FSGet / List→FSList / Stat→FSStat / Set,Mkdir→FSSuccess）
   → SendSyscallData(reply_to, ...) で要求元起床
```

## 設計原則
- **カーネル完全ステートレス**: pending 表・待機リスト・rep_ch レジストリなし。状態は Virt ノード・チャネル(callee_pid)・メッセージ内のみ
- **宛先は requested_by から導出**: reply_to は Provider が requested_by をエコーしたもの（宛先 pid そのもの）。相関トークン不要（lock-step）
- **ReqCh の caller_pid = 0（カーネル）**で要求元識別には無意味。requested_by のみが権威（quirks #4 と一致）
- **待機は既存機構流用**: WaitingForEvent + SendSyscallData。新リスト不要
- **横取りは callee_pid[0]**: レジストリ不要。pid 0 = ただの購読者（GetPid/createVFS の pid 0 慣習と整合）
- 複数プロセスの同一ノード保留 OK（reply_to で区別）
- Provider 死亡は Req 配達時に検知（req_ch 購読者非生存なら Fail 合成）。Req 受領後の死亡はハング（将来の安全機構）

## 既知リスク（重要度順）
1. **アイドルリセットとの相互作用**: 委譲チェーン全員が WaitingForEvent になると `processes.clear()` で全滅（ハングではなく消滅）。検証時は keepalive プロセス必須。将来 idle 判定見直し
2. **Sub の逆流は未定義**: 初期版は Sub 委譲を Fail にする（Pub は同期応答で委譲可）
3. **Provider 途中死亡**: ハング＋アイドルリセットと合流し得る
4. **reply_to 偽装**: Provider が任意 pid へ型付き応答を注入可能（協調前提で受容）
5. **TS Process との未検証**: カーネル直接 WaitingForEvent 設定と TS トークン処理の噛み合わせ
6. 小規模未確定: Req ペイロードの FSObj 表現 / Virt 自体の list・stat / Get(/v1) で Virt 自体が取得可能

## 前提・周辺
- Overlay は本機構の上の userland Provider として実装（design/overlay-fs の意味論: union・Upper 書込）
- 現行 Object enum へ `Virt{req_ch, rep_ch}` 追加。kernel.rs の Virt 検知は match か FSFrontend か（実装判断）
- GetPid 実装済み。pid 0 はカーネル予約

## 未決
- Sub/Pub 逆流方式（将来）
- 安全機構: 保留必須化・Provider 途中死亡タイムアウト
