mod generator;
mod processes;

extern crate ghs_demangle;

pub use generator::generate_user_id;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn demangle_str(x: String) -> String {
    ghs_demangle::demangle(x).to_string()
}
