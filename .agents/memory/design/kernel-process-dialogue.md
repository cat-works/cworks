# Kernel-Process Dialogue（2026-08・vObj 限定・初期版確定）

## スコープ明確化（ユーザー確定）
- **inbox / 一般メッセージ / シグナルはスコープ外**（将来の別テーマ）。vObj は専用チャネルで完結

## vObj 構成（RepCh 復活）
- **`Object::Virt { req_ch, rep_ch }`**（二本）
- Req: kernel → Provider（req_ch の Invoke、payload = `Req{requested_by, method, path, content}`）
- Res: Provider → rep_ch へ publish（`Res{reply_to, content}`）
- kernel: rep_ch への publish を検知 → pending から型合成 → 要求元 outgoing へ配送

## 宛先特定・要求元識別（ユーザーの発見）
- **`reply_to` は requested_by から導出**（Provider が受信 Req の requested_by をエコー）。不透明な相関トークンではなく宛先 pid そのもの（send_to と一致）
- **複数プロセスが同一 Virt ノードへ保留しても、reply_to(=要求元 pid) で区別できる** → 「ノードあたり 1 保留」制約は不要
- **ReqCh の Invoke 元は Kernel (Pid 0)** → caller_pid は要求元識別に使えない。**requested_by のみが権威**（quirks #4 の教訓と一致）
- pending 表: `HashMap<requester_pid, {method, path, node}>`（lock-step で要求元あたり 1 件）

## 型合成マップ（method → Reply）
Get→FSGet / List→FSList / Set,Mkdir→FSSuccess / Stat→FSStat / Sub→FSSuccess

## 初期版方針
1. Provider 死亡は Req 配達時に検知（req_ch 購読者非生存なら Fail 合成）
2. 応答ルーティングは reply_to(=requested_by) で完結

## エッジケース（初期版）
| ケース | 挙動 |
|--------|------|
| Req 配達時 Provider 非生存 | 即エラー応答（Fail 合成） |
| Provider が Req 受領後・応答前で死亡 | 依然ハング（将来の安全機構） |
| Provider 自己再帰 | 可。循環チェーンは文書化して受容 |
| Sub の仮想ノード経由（逆流） | 応答は型合成で即返し、実イベントは provider が要求元へリレー（詳細未定） |

## 前提・周辺
- VFS 設計（design/overlay-fs）: Provider は普通の syscall で Overlay 解決を行う userland プロセス
- 現行 Object enum（object.rs）: Int/String/Boolean/Float/Double/Bytes/Null/CompoundFSObj/Func → `Virt{req_ch, rep_ch}` 追加予定
- kernel.rs step() は fs_frontend を毎 step 構築して match。Virt 検知は kernel.rs の match か FSFrontend.resolve のどちらか（実装判断）
- GetPid 実装済み。pid 0 はカーネル予約

## 未決
- Sub/Pub の仮想ノード経由の逆流方式（詳細未定）
- 将来の安全機構: 保留必須化・Provider 途中死亡タイムアウト
