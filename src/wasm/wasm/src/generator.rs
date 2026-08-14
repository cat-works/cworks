use chrono::{DateTime, Local};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[must_use]
pub fn generate_user_id() -> String {
    let dt: DateTime<Local> = Local::now();
    let mut timestamp: i64 = dt.timestamp();

    let mut id = String::new();
    while timestamp > 0 {
        #[allow(clippy::cast_sign_loss)]
        let digit = (timestamp % 64) as u8;

        let digit = if digit < 10 {
            (digit + 48) as char
        } else if digit < 10 + 26 {
            (digit + 55) as char
        } else if digit < 10 + 26 * 2 {
            (digit + 61) as char
        } else if digit == 62 {
            '_'
        } else {
            '-'
        };

        id.push(digit);
        timestamp >>= 6;
    }

    id = id.chars().rev().collect();
    id
}
