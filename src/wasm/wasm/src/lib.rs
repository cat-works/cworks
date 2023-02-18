mod generator;
mod js_process;
mod processes;
mod session;

extern crate ghs_demangle;

pub use generator::generate_user_id;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);

    #[wasm_bindgen(js_namespace = console, js_name = clog)]
    fn log2(a: JsValue);

    #[wasm_bindgen(js_name = my_stringify)]
    fn stringify(a: JsValue) -> JsValue;
}
/*
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
} */

#[wasm_bindgen]
pub fn demangle_str(x: String) -> String {
    ghs_demangle::demangle(x).to_string()
}

#[wasm_bindgen(start)]
fn m() {
    // let s = serde_wasm_bindgen::to_value(&PollResult::<i64>::Syscall(Syscall::Send(
    //     HandleIssuer::default().get_new_handle(0, HandleData::None),
    //     "aiu".to_string(),
    // )));
    // log2(s.unwrap());
    // console_log!("{:?}", s.unwrap());
}
