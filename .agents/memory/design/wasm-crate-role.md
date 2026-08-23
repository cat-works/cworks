# wasm クレートの役割設計（2026-08・最終確定・成果物フリップ完了見込み）

## 最終アーキテクチャ（案 C / RustPython 方式）

| ディレクトリ | パッケージ名 | 役割 |
|--------------|--------------|------|
| `wasm/wasm` | **cworks** | 成果物クレート（cdylib）。全 `__ffi_*` エクスポート・glue JS・build.rs・pkg 出力＝JS 境界の一切 |
| `wasm/lua` | **cworks-lua** | 純粋な mlua ラッパー（rlib）。FFI エクスポートなし・ワイヤ形式（NUL エスケープ）を知らない |
| `wasm/kernel` | kernel | 変更なし |
| `wasm/test-cli` | test-cli | 変更なし |

依存 DAG: `{kernel, cworks-lua} ← cworks`（一方向）

## 設計決定の経緯と根拠

1. **当初案 A**（wasm=機械的輸送層、成果物は lua 残置）→ ユーザーの方向転換で発展解消: 「lua を純粋な mlua ラッパーに」「将来的に lua crate は削除」
2. **S-1 スパイク**: 依存 build.rs の `cargo:rustc-link-arg` は最終 emcc リンクに伝播しない（`links` キー併用も不成立を実測）→ **build.rs はリンク側クレートに置くしかない → build.rs を wasm に置く ⟺ 成果物が wasm**
3. **S-2 スパイク**: rustc emscripten リンカは依存 rlib 内の `no_mangle` シンボルも自動 export（`__spike_probe` で実測）→ 明示的 EXPORTED_FUNCTIONS 管理不要
4. **buf_encoding の配置**: C 文字列 NUL 打ち切り対策（Uint8Array 転送時代からの系譜）で、境界を越えるのは shim 関数内のみ → **FFI 境界（wasm crate）に配置**。lua 側メソッドは素の String で流通。エラー時センチネル `\x01\x03` の生成も shim へ移動
5. **cworks-lua 公開 API から mlua 型を漏らさない**: `LuaThreadError::Mlua(String)` 化。成果物側は mlua 直接依存しない

## 実装詳細

- 移動: js_callback / buf_encoding(+tests) / ffi_session / demangle(__ffi_demangle) / __ffi_init / glue JS ×2 / build.rs
- 新規分離: `ffi_luaenv.rs` / `ffi_luathread.rs`（ポインタ Box 化＋CString 整形＋encode/decode 適用点）
- 型付き API 化: `LuaEnv` pub フィールド廃止・`thread(name, code)` メソッド化（`.expect` パニック挙動維持）、`yield_process()` pub 化
- シンボル名は全て不変（`__ffi_*`, `cworks_js_callback`）→ TS ラッパ内容無変更、フロントは import パス ×5 のみ更新
- ハウスキーピング: `wasm/lua/{.cargo,.vscode}` 削除（ユーザー承認済み）

## 将来構想: CLI フロントエンド（2026-08・構想段階・着手未定）

- **目的**: ブラウザ経由で可能な OS 操作の**全て**（FS 操作含む）を CLI からカバー
- **先例**: SQLite — コア 1 つに対し sqlite3 CLI と WASM モジュールという 2 フロントエンド
- **実現性**: kernel/test-cli がネイティブ動作証明済み。mlua はネイティブコンパイル可。差分は (1) syscall 輸送 (2) stdio 端点 (3) パッケージング のみ
- **CLI 着手時の必須工程（先送り確定）**: `.lua` パーソナリティ資産（`src/lib/lua/*.lua`）のワークスペース共有化（`include_str!`、フロント `?raw` import に影響）。アイドルリセット対策（常時 Running のホスト側プロセス）
- **留意**: JS ブリッジ（本クレート）はブラウザ専用であり CLI では再利用されない。分離の価値は可読性と境界明示

## 関連メモリ
- `design/cargo-workspace-layout` — レイアウト・命名
- `design/cworks-runtime-quirks` — カーネル特性・JSON 境界問題

## 実装完了（2026-08・成果物フリップ）

全フェーズ実装・検証済み:

### 最終状態
- `wasm/wasm` = **cworks**（cdylib+rlib）: `src/{lib.rs(__ffi_init+__ffi_demangle), ffi_session, ffi_luaenv, ffi_luathread, js_callback, buf_encoding}` + `glue/{callback-pre,cworks-lib}.js` + `build.rs` + `build.sh` + `pkg/{cworks.ts, .gitignore}`
- `wasm/lua` = **cworks-lua**（rlib のみ, deps mlua+log）: `luaenv.rs`（LuaEnv 構造体＋new/run_code/thread メソッド・pub フィールド廃止）、`luathread.rs`（yield_process pub 化、LuaThreadError::{Mlua(String), InvalidData}）
- 依存 DAG: `{kernel, cworks-lua} ← cworks`
- フロント import ×5 を `wasm/wasm/pkg/cworks` へ repoint。シンボル名は全て不変

### 実装時の確定事項
- `LuaEnv::thread(name, code)`: コルーチン生成ロジック（wrapped_code 構築〜eval）を旧 `__ffi_luaenv_thread` から取り込み。`.expect("Failed to create coroutine")` パニック挙動は現行維持
- センチネル `\x01\x03` の生成とエラーログは shim 側に移動。InvalidData 時の "Expected a string..." ログは cworks-lua 内に残留
- 旧 `wasm/lua/pkg/lua.ts`（統合前の死蔵ラッパ・参照ゼロ）は pkg 廃止時に削除
- exports 総数 29（フリップ前 30 から -1）。必須 FFI 11 種は全て存在を node で照合済み。差分 1 は emscripten 内部 export の揺れと判断
- glue が build.rs と同一パッケージ内になったため rerun-if-changed 追加は不要（デフォルト監視で再リンクされる）

### 検証結果（全 PASS）
cargo check --workspace / cargo test -p cworks (buf_encoding 2 tests) / pnpm run wasm-lua（20.1MB, glue 焼き込み確認）/ svelte-check 関連エラーなし / CDP 回帰: debug シェル ls×2 プロンプト復帰 ✓、ghs-demangler foo__Fvi→foo(void,int) ✓、ページエラー 0

### 残課題
- yield エラー経路（LuaThreadError→センチネル）は E2E happy-path で踏まないため、コードレビューでの確認のみ（実施済み・挙動一致）
