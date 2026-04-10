//! iOS FFI 인터페이스 — Swift에서 Rust rhwp 엔진 호출
//!
//! C-ABI 함수로 HWP 파일 로드, 페이지 수 조회, SVG 렌더링을 제공한다.
//! iOS 네이티브 앱(알한글)에서 사용.

use std::ffi::{c_char, CString};
use std::ptr;

use crate::wasm_api::HwpDocument;

/// 불투명 핸들 (Swift에서 포인터로 전달)
pub struct RhwpHandle {
    doc: HwpDocument,
}

/// HWP 파일 데이터를 로드하여 핸들을 반환한다.
/// 실패 시 null 반환.
#[no_mangle]
pub extern "C" fn rhwp_open(data: *const u8, len: usize) -> *mut RhwpHandle {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    match HwpDocument::from_bytes(bytes) {
        Ok(doc) => Box::into_raw(Box::new(RhwpHandle { doc })),
        Err(_) => ptr::null_mut(),
    }
}

/// 문서의 페이지 수를 반환한다.
#[no_mangle]
pub extern "C" fn rhwp_page_count(handle: *const RhwpHandle) -> u32 {
    if handle.is_null() {
        return 0;
    }
    let h = unsafe { &*handle };
    h.doc.page_count()
}

/// 특정 페이지를 SVG 문자열로 렌더링한다.
/// 호출자는 반환된 문자열을 `rhwp_free_string`으로 해제해야 한다.
/// 실패 시 null 반환.
#[no_mangle]
pub extern "C" fn rhwp_render_page_svg(handle: *const RhwpHandle, page: u32) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }
    let h = unsafe { &*handle };
    match h.doc.render_page_svg_native(page) {
        Ok(svg) => match CString::new(svg) {
            Ok(cstr) => cstr.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

/// `rhwp_render_page_svg`가 반환한 문자열을 해제한다.
#[no_mangle]
pub extern "C" fn rhwp_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}

/// 문서 핸들을 해제한다.
#[no_mangle]
pub extern "C" fn rhwp_close(handle: *mut RhwpHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)); }
    }
}
