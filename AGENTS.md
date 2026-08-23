# cworks プロジェクトのエージェント指示

## ビルド環境

### 必ず `nix develop --command` 経由でビルドすること

opencode セッションから直接 `cargo build --target wasm32-unknown-emscripten` を実行すると、emscripten ツールチェーンが正しく動作せずリンクエラー（`__cxa_find_matching_catch_3` 未定義等）が発生する。

正しいビルドコマンド:
```bash
nix develop --command bash wasm/lua/build.sh
# または
nix develop --command pnpm run wasm-lua
```

直接実行すると失敗する理由:
- opencode セッションの PATH に nix のツールチェーンが正しく含まれていない
- emscripten のシステムライブラリのビルドが不完全になる
- `TARGET_CC=emcc` 等のクロスコンパイル環境変数が未設定

**これは emscripten のバージョンやコードの問題ではなく、シェル環境の問題である。**
