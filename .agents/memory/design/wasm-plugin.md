# Wasm Plugin 設計（2026-08・最終状態）

## アーキテクチャ
```
kernel → Process trait (poll: SyscallData → PollResult)
  └─ RustProcessCore (ProcessSession + data_handlers) ← ネイティブ Rust プロセス
  └─ Session (RustProcessCore エイリアス)            ← WASM プラグイン

Plugin = 自己完結 wasm モジュール (wasm32-unknown-unknown, std 許可)
  ├─ poll プロトコル: kernel が SyscallData を供給 → プログラムが PollResult を返す
  ├─ ABI: postcard（SyscallData / PollResult 直行・JSON 輸送不要）
  └─ エクスポート: cworks_alloc / cworks_plugin_poll（u32-LE 長さプレフィックス + postcard）
```

## cworks-sdk（SDK パッケージ）
| 機能 | 内容 |
|------|------|
| `Session` | kernel::RustProcessCore のエイリアス（同一型） |
| `#[cworks_sdk::plugin]` | proc-macro: エクスポート生成 + async エントリポイント |
| `step_plugin` | ホスト側ドライバ: pass_syscall_data → poll → take_syscall + postcard エンコード |
| `encode/decode_request/response` | wire ABI ヘルパ（u32-LE プレフィックス + postcard） |
| `prelude` | DataHandler / FSObjRef / Object / ProcessClientExt / Session / SyscallError |

## プラグインの書き方
```rust
#[cworks_sdk::plugin]
async fn main(mut session: Session) {
    let handler: DataHandler = Rc::new(Box::new(|data: Option<FSObjRef>| {
        // data から処理...
        Ok(())
    }));
    session.subscribe("/srv/eval/req".to_string(), handler).await?;
    loop { session.wait_for_event().await?; }
}
```

## 現在のワークスペース
wasm/{ kernel, lua(=cworks-lua), sdk, sdk-macros, test-cli, plugin-eval, wasm(=cworks artifact) }

## 関連
- design/overlay-fs — 仮想 FS（将来 vObj プロバイダの基礎）
- design/kernel-process-dialogue — send モデル・Req/Res
- conventions/cworks-coding-style（global）
