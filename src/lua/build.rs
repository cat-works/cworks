use std::path::Path;

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target != "wasm32-unknown-emscripten" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let pkg_dir = Path::new(&manifest_dir).join("pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    let js = pkg_dir.join("cworks-rs.js");
    let tsd = pkg_dir.join("cworks-rs.d.ts");
    let pre_js = Path::new(&manifest_dir).join("callback-pre.js");
    let js_lib = Path::new(&manifest_dir).join("cworks-lib.js");

    println!("cargo:rustc-link-arg=-sSIDE_MODULE=0");
    println!("cargo:rustc-link-arg=-o{}", js.display());
    println!("cargo:rustc-link-arg=--emit-tsd");
    println!("cargo:rustc-link-arg={}", tsd.display());
    println!("cargo:rustc-link-arg=-sMODULARIZE=1");
    println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
    println!("cargo:rustc-link-arg=-sMAIN_MODULE=2");
    println!("cargo:rustc-link-arg=--pre-js={}", pre_js.display());
    println!("cargo:rustc-link-arg=--js-library={}", js_lib.display());
    println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
    println!(
        "cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall,UTF8ToString,stringToUTF8,lengthBytesUTF8"
    );
}
