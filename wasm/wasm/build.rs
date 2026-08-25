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
    let pre_js = Path::new(&manifest_dir).join("glue/callback-pre.js");
    let js_lib = Path::new(&manifest_dir).join("glue/cworks-lib.js");

    // Calling function that have string arguments from JS requires ccall (or UTF8ToString/stringToUTF8)
    println!(
        "cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall,cwrap,UTF8ToString,stringToUTF8,lengthBytesUTF8"
    );

    // Vite works with ES6 modules, so we need MODULARIZE=1 and EXPORT_ES6=1
    println!("cargo:rustc-link-arg=-sMODULARIZE=1");
    println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");

    // the crate being compiled as a wasm library
    println!("cargo:rustc-link-arg=-o{}", js.display());

    println!("cargo:rustc-link-arg=--emit-tsd={}", tsd.display());
    println!("cargo:rustc-link-arg=--pre-js={}", pre_js.display());
    println!("cargo:rustc-link-arg=--js-library={}", js_lib.display());
}
