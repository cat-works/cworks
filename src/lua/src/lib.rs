pub mod buf_encoding;
pub mod ffi_session;
pub mod js_callback;
pub mod luaenv;
pub mod luathread;

#[unsafe(no_mangle)]
pub extern "C" fn __ffi_init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    log::info!("CWorks module initialized.");
}
