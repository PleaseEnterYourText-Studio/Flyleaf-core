use std::ffi::{c_char, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::model::Clippings;

pub const ABI_VERSION: u32 = 1;

#[no_mangle]
pub extern "C" fn flyleaf_core_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn flyleaf_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn flyleaf_parse_clippings(data: *const u8, len: usize) -> *mut c_char {
    let payload = catch_unwind(AssertUnwindSafe(|| -> String {
        let result = parse_impl(data, len);
        match result {
            Ok(clippings) => serde_json::json!({ "ok": true, "data": clippings }).to_string(),
            Err(error) => error_envelope(error.code(), &error.to_string()),
        }
    }))
    .unwrap_or_else(|_| error_envelope("panic", "核心库内部错误"));

    CString::new(payload)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn flyleaf_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

fn parse_impl(data: *const u8, len: usize) -> crate::Result<Clippings> {
    if data.is_null() {
        return Err(crate::Error::InvalidData("输入指针为空".into()));
    }
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    crate::parse::parse_clippings(bytes)
}

fn error_envelope(code: &str, message: &str) -> String {
    serde_json::json!({
        "ok": false,
        "error": { "code": code, "message": message }
    })
    .to_string()
}
