//! iOS FFI 인터페이스 — Swift에서 Rust rhwp 엔진 호출
//!
//! C-ABI 함수로 HWP 파일 로드, 페이지 수 조회, SVG 렌더링을 제공한다.
//! iOS 네이티브 앱(알한글)에서 사용.

use std::ffi::{c_char, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use crate::wasm_api::HwpDocument;

/// 패닉 가능성이 있는 FFI 함수에서 사용하는 가드 매크로.
/// null 핸들 체크 + catch_unwind로 패닉이 C-ABI 경계를 넘지 못하게 방어한다.
macro_rules! ffi_guard {
    ($handle:expr, $default:expr, $body:expr) => {{
        if $handle.is_null() {
            return $default;
        }
        match catch_unwind(AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(_) => $default,
        }
    }};
}

/// 불투명 핸들 (Swift에서 포인터로 전달)
pub struct RhwpHandle {
    doc: HwpDocument,
}

/// 페이지 크기 (포인트 단위). Swift에서 C 구조체로 직접 접근.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RhwpPageSize {
    pub width_pt: f64,
    pub height_pt: f64,
}

/// HWP 파일 데이터를 로드하여 핸들을 반환한다.
/// 실패 시 null 반환.
#[no_mangle]
pub extern "C" fn rhwp_open(data: *const u8, len: usize) -> *mut RhwpHandle {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let bytes = unsafe { std::slice::from_raw_parts(data, len) };
        match HwpDocument::from_bytes(bytes) {
            Ok(doc) => Box::into_raw(Box::new(RhwpHandle { doc })),
            Err(_) => ptr::null_mut(),
        }
    }));
    result.unwrap_or(ptr::null_mut())
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

/// 특정 페이지의 크기를 포인트 단위로 반환한다.
/// 실패 시 (0.0, 0.0) 반환.
#[no_mangle]
pub extern "C" fn rhwp_page_size(handle: *const RhwpHandle, page: u32) -> RhwpPageSize {
    const ZERO: RhwpPageSize = RhwpPageSize { width_pt: 0.0, height_pt: 0.0 };
    ffi_guard!(handle, ZERO, {
        let h = unsafe { &*handle };
        let json = match h.doc.get_page_info_native(page) {
            Ok(j) => j,
            Err(_) => return ZERO,
        };
        // get_page_info_native가 반환하는 JSON에서 width, height 추출
        // JSON 파서 의존성 없이 간단 파싱
        let width = extract_json_f64(&json, "width").unwrap_or(0.0);
        let height = extract_json_f64(&json, "height").unwrap_or(0.0);
        RhwpPageSize { width_pt: width, height_pt: height }
    })
}

/// 특정 페이지를 SVG 문자열로 렌더링한다.
/// 호출자는 반환된 문자열을 `rhwp_free_string`으로 해제해야 한다.
/// 실패 시 null 반환.
#[no_mangle]
pub extern "C" fn rhwp_render_page_svg(handle: *const RhwpHandle, page: u32) -> *mut c_char {
    ffi_guard!(handle, ptr::null_mut(), {
        let h = unsafe { &*handle };
        match h.doc.render_page_svg_native(page) {
            Ok(svg) => match CString::new(svg) {
                Ok(cstr) => cstr.into_raw(),
                Err(_) => ptr::null_mut(),
            },
            Err(_) => ptr::null_mut(),
        }
    })
}

/// 특정 페이지의 렌더 트리를 JSON 문자열로 반환한다.
/// 호출자는 반환된 문자열을 `rhwp_free_string`으로 해제해야 한다.
/// 실패 시 null 반환.
#[no_mangle]
pub extern "C" fn rhwp_render_page_tree(handle: *const RhwpHandle, page: u32) -> *mut c_char {
    ffi_guard!(handle, ptr::null_mut(), {
        let h = unsafe { &*handle };
        match h.doc.build_page_render_tree(page) {
            Ok(tree) => {
                match serde_json::to_string(&tree.root) {
                    Ok(json) => match CString::new(json) {
                        Ok(cstr) => cstr.into_raw(),
                        Err(_) => ptr::null_mut(),
                    },
                    Err(_) => ptr::null_mut(),
                }
            }
            Err(_) => ptr::null_mut(),
        }
    })
}

/// 문서 내 이미지 바이너리 데이터를 반환한다.
/// bin_data_id는 1-indexed. 반환된 포인터는 핸들이 유효한 동안만 사용 가능.
/// Swift 측에서 Data(bytes:count:)로 복사하여 사용할 것.
/// 실패 시 null 반환, out_len은 0으로 설정.
#[no_mangle]
pub extern "C" fn rhwp_image_data(
    handle: *const RhwpHandle,
    bin_data_id: u16,
    out_len: *mut usize,
) -> *const u8 {
    if handle.is_null() || out_len.is_null() || bin_data_id == 0 {
        if !out_len.is_null() {
            unsafe { *out_len = 0; }
        }
        return ptr::null();
    }
    let h = unsafe { &*handle };
    let idx = (bin_data_id - 1) as usize;
    match h.doc.get_bin_data(idx) {
        Some(data) => {
            unsafe { *out_len = data.len(); }
            data.as_ptr()
        }
        None => {
            unsafe { *out_len = 0; }
            ptr::null()
        }
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

/// JSON 문자열에서 특정 키의 f64 값을 추출하는 간이 파서.
/// serde_json 의존성 없이 `"key":123.4` 패턴을 찾는다.
fn extract_json_f64(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| c == ',' || c == '}' || c == ' ')?;
    rest[..end].parse::<f64>().ok()
}
