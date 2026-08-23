# Cargo workspace 構成（2026-08・最終・ユーザー指定レイアウト）

`src/wasm` / `src/lua` は廃止。Rust コードは `(pj)/wasm/` 配下の cargo workspace（仮想マニフェスト）で管理。

## レイアウト（ユーザー確定）
```
wasm/
├── Cargo.toml      # ワークスペースルート (仮想マニフェスト) + [profile.release]
├── kernel/         # カーネルライブラリ (プラットフォーム中立)
├── lua/            # パッケージ名 cworks-lua: 純粋な mlua ラッパー rlib（FFI エクスポートなし）
├── test-cli/       # bin ターゲット: 機能試験 playground
└── wasm/           # パッケージ名 cworks: 成果物クレート (emscripten cdylib, build.rs/glue/pkg)
```

## パッケージ命名（2026-08・成果物フリップ時に確定）
- 成果物 = `cworks`（ディレクトリ `wasm/wasm`）。`pnpm run wasm-lua` / `-p cworks` はこの名前を使う
- ラッパー = `cworks-lua`（ディレクトリ `wasm/lua`）。パッケージ名 ≠ ディレクトリ名に注意

## 運用ルール
- **WASM ビルドは必ず `-p cworks`**: `cargo build --target wasm32-unknown-emscripten --release -p cworks`（build.sh 反映済み）。
- **playground**: `nix develop --command bash -c "cd wasm && cargo run -p test-cli"`。
- **--emit-tsd は tsc 必要**: `nix develop --command pnpm run wasm-lua` 経由で実行すること。

## Vite fs.allow（重要な落とし穴）
- フロントは `wasm/wasm/pkg/*` を直接 import するため、**Vite の server.fs.allow にプロジェクトルートが必要**。未設定だと `/wasm/...` が 403 → ページ全体が 500 になる。
- 対策済み: vite.config.ts の `server.fs.allow = [searchForWorkspaceRoot(process.cwd())]`。

## 関連メモリ
- `design/wasm-crate-role` — クレート役割設計・フリップ詳細
