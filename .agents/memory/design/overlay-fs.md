# Overlay FS 設計（2026-08・確定： 仮想ノード機構 + Userland 実装）

## 最終方針（ユーザー確定）
- **kernel = 機構のみ**: 汎用仮想 FS ノード `FSObject::Virtual` を追加し、意味論は外部ライブラリ実装に委ねる（mechanism/policy 分離・FUSE 型構図）
- **Overlay は Userland ライブラリ**: kernel は Overlay 特有の意味論を一切持たない
- **プロセス非依存**: 同一 syscall は同一結果（per-pid コンテキスト案は不採用）
- 委譲方式は **(i) 同一アドレス空間ハンドラ登録**（別プロセス/JS 委譲は poll 内再帰の危険により不採用）

## 解決規則（Overlay ライブラリの意味論）
- 成分単位のパス解決: 各成分で Upper 存在→Upper / なければ Lower フォールスルー
- ディレクトリは UNION（子名前空間の合併・同名は Upper 優先）。葉は丸ごと shadow
  - ※ エントリ単位丸ごと判定だと「既存ディレクトリへ 1 ファイル追加」が下位層を隠して破綻する（/usr/bin/newcmd 問題）
- 書き込みは常時 Upper・親ディレクトリ暗黙生成。Lower は不変（Set 全値置換なので copy-up 相当が自動達成）

## 実装形状: 形 B「候補パス列挙型」（推奨・シンプル第一）
```rust
// kernel 側トレイト（機構）
trait VirtualFs {
    fn candidates(&self, rel: &Path) -> Vec<Path>; // 優先順 [upper, lower]
    fn write_target(&self, rel: &Path) -> Path;
}
```
- ライブラリは純パス計算のみ（木に触れない）→ unit test 自明
- kernel 側汎用手続き: 存在確認=候順フォールスルー / list=優先順付きマージ / write=write_target
- 代替 形 A（view 注入の操作委譲）は柔軟だが借用・再入の配慮が必要なため不採用

## 例（意味論確認済み）
/up:{a}, /lo:{b/{c}}, /ov:Ov{Upper:/up,Lower:/lo} において:
list(/ov)={a,b} / get(/ov/a)=/up/a / get(/ov/b/c)=/lo/b/c（成分単位多段フォールスルー）/ set(/ov/b/z)=/up/b/z 作成 / set(/ov/b/c)=shadow（/lo/b/c は直接パスで残存）

## 文脈変数への適用
- stdin/stdout/editor は対象スコープの Upper 層にエントリ（Channel）として植える。チャネルは subscribe 自動生成を利用可能
- 「継承しつつ部分変更」= 既存 Overlay を Lower とする新 Overlay の作成（チェーン・入れ子は同一規則の自然な再帰）
- カーネル・cworks.* API・stdio.lua・Native LuaProcess 無変更。exec-lua の stdout/stdin 必須バリデーション等のボイラープレート解消
- CLI の TTY も同一機構

## 未決・実装時確定事項
- トレイト名・フィールド命名（Upper/Lower 採用方向。Rust 予約語注意）
- Subscribe/Publish の扱い: 仮想ノードは「解決して実ノードを返す」のみにし、チャネル操作は実ノード直行とする割り切り案（チャネル自動生成と整合）
- 循環ガード（作成時検証＋解決深度上限）、Overlay ノード自体の Get/Stat 可視性、ハンドラ登録/解除（unmount）API
- セッション Upper 層の掃除は現行 /run 同様手動（将来プロセス連動 GC 検討）

## 関連
- `design/wasm-crate-role` — Native LuaProcess アーキテクチャ
- `design/cworks-runtime-quirks` — LIFO 配送等のカーネル特性
