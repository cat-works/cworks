mod generator;
mod js_process;
mod session;

extern crate ghs_demangle;

pub use generator::generate_user_id;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn demangle_str(x: String) -> String {
    ghs_demangle::demangle(x).to_string()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

    log::debug!("Debug");
    log::info!("Info");
}
