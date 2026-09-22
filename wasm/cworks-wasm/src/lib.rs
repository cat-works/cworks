pub mod bindings;

use wasm_bindgen::prelude::*;

mod console {
    use log::Level;

    #[cfg(target_os = "emscripten")]
    pub fn write(level: Level, msg: &str) {
        use std::ffi::{CString, c_char};

        unsafe extern "C" {
            fn emscripten_console_log(utf8_string: *const c_char);
            fn emscripten_console_warn(utf8_string: *const c_char);
            fn emscripten_console_error(utf8_string: *const c_char);
        }

        let c = CString::new(msg).unwrap();
        unsafe {
            match level {
                Level::Error => emscripten_console_error(c.as_ptr()),
                Level::Warn => emscripten_console_warn(c.as_ptr()),
                _ => emscripten_console_log(c.as_ptr()),
            }
        }
    }

    #[cfg(not(target_os = "emscripten"))]
    pub fn write(level: Level, msg: &str) {
        let msg = msg.replace('\n', "\\n");
        match level {
            Level::Error | Level::Warn => eprintln!("{msg}"),
            _ => println!("{msg}"),
        }
    }
}

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let msg = format!("[{}] {}", record.target(), record.args()).replace('\n', "\\n");
        console::write(record.level(), &msg);
    }

    fn flush(&self) {}
}

static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

#[wasm_bindgen(start)]
pub fn start() {
    let _ = log::set_logger(&CONSOLE_LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}
