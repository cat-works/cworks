use std::path::Path;

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target != "wasm32-unknown-emscripten" {
        return;
    }

    let pkg_dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    let js = pkg_dir.join("lua-rs.js");
    let tsd = pkg_dir.join("lua-rs.d.ts");

    // rustc は cdylib を wasmのみの SIDE_MODULE としてリンクするため、
    // emscripten の JS グルー(lua-rs.js) と型定義(lua-rs.d.ts) を生成できない。
    // 後置された link-arg で出力先を lua-rs.js に上書きし、MAIN_MODULE としてリンクする。
    println!("cargo:rustc-link-arg=-sSIDE_MODULE=0");
    println!("cargo:rustc-link-arg=-o{}", js.display());
    println!("cargo:rustc-link-arg=--emit-tsd");
    println!("cargo:rustc-link-arg={}", tsd.display());
    println!("cargo:rustc-link-arg=-sMODULARIZE=1");
    println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
    println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall");
    println!("cargo:rustc-link-arg=-sMAIN_MODULE=2");
}
