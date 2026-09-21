use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe extern "C" {
    fn cworks_js_callback(id: i32, data_ptr: *const u8, data_len: i32) -> i32;
}

pub fn call_js_callback(id: i32, data_json: &str) -> Option<String> {
    let c_data = CString::new(data_json).ok()?;
    let ptr = c_data.as_bytes().as_ptr();
    let len = c_data.as_bytes().len() as i32;
    let result_ptr = unsafe { cworks_js_callback(id, ptr, len) };
    if result_ptr < 0 {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(result_ptr as *const c_char) };
    Some(c_str.to_string_lossy().into_owned())
}
