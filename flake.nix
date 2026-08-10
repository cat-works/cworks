{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        # nixpkgs は CC=gcc を設定するため、そのままでは cc crate が
        # emscripten 向けクロスビルドでも gcc を使ってしまう。
        # TARGET_CC はクロスビルド時にのみ参照される（ホストビルドは CC を使う）。
        env = {
          TARGET_CC = "emcc";
          TARGET_CXX = "em++";
        };
        buildInputs = [
          pkgs.nodejs_26
          pkgs.pnpm

          (pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
            targets = [
              "wasm32-unknown-unknown"
              "wasm32-unknown-emscripten"
            ];
          })
          pkgs.rust-analyzer
          pkgs.clang
          pkgs.rustfmt
          pkgs.rustc
          pkgs.clippy
          pkgs.rust-cbindgen
          pkgs.emscripten
        ];
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      };
    };
}
