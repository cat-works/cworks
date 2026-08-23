# Cargo workspace 構成（2026-08・最終・ユーザー指定レイアウト）

`src/wasm` / `src/lua` は廃止。Rust コードは `(pj)/wasm/` 配下の cargo workspace（仮想マニフェスト）で管理。

## レイアウト（ユーザー確定）
```
wasm/
├── Cargo.toml      # ワークスペースルート (仮想マニフェスト) + [profile.release]
├── kernel/         # カーネルライブラリ (プラットフォーム中立)
├── lua/            # cworks パッケージ (emscripten cdylib, build.rs/build.sh/callback-pre.js/cworks-lib.js/pkg)
├── test-cli/       # bin ターゲット: 機能試験 playground
└── wasm/           # 旧 wasm-bindgen クレート (Session + demangle_str, wasm-pack 用)
```

## 運用ルール
- **WASM ビルドは必ず `-p cworks`**: `cargo build --target wasm32-unknown-emscripten --release -p cworks`（build.sh 反映済み）。bin メンバー巻き込み防止。
- **playground**: `nix develop --command bash -c "cd wasm && cargo run -p test-cli"`。pub/sub + fs サンプル付き。
- **--emit-tsd は tsc 必要**: `nix develop --command pnpm run wasm-lua` 経由で実行すること。

## Vite fs.allow（重要な落とし穴）
- フロントは `wasm/lua/pkg/*` を直接 import するため、**Vite の server.fs.allow にプロジェクトルートが必要**。未設定だと `/wasm/lua/pkg/*` が 403 → ページ全体が 500 になる。
- 対策済み: vite.config.ts の `server.fs.allow = [searchForWorkspaceRoot(process.cwd())]`。新規に src 外から成果物を import する際は同様の注意。

## demangle_str の統合（2026-08・完了）
- `demangle_str` (GHS デマングル) を統合 emscripten モジュールへ port。`src/lib/index.ts` は旧 wasm-pack 成果物に非依存 (`export { demangle_str as ghs_demangle } from "../../wasm/lua/pkg/cworks"`)。
- Rust 側: `__ffi_demangle(*const c_char) -> *mut c_char`、`CString::into_raw()` 慣習（yield と同様のリーク許容）。EXPORTED_FUNCTIONS は rustc が no_mangle シンボルから自動生成のため build.rs 変更不要。
- 検証テストベクトル: `foo__Fvi` → `foo(void, int)`。**`_Z3fooi` 等の Itanium 形式は GHS 形式でないため素通しが正しい挙動**。
- `wasm/wasm/` クレートはソース保存のみ残置（ビルド不要）。

