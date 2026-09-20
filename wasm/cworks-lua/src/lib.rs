//! Pure mlua wrapper library for `CWorks`.
//!
//! This crate exposes typed Rust APIs (`LuaEnv`, `LuaThread`) over mlua and
//! knows nothing about the JS boundary: no FFI exports, no wire encoding.
//! All C-ABI marshaling lives in the artifact crate (`cworks`).
//!
//! The public API must not leak mlua types; errors are converted to
//! self-contained values at construction.

mod lua_process;

pub use lua_process::new_lua_process;
