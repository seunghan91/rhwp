# Task #90: Rust aarch64-apple-ios 크로스 컴파일 — 구현 계획서

## 현재 아키텍처

### Cargo.toml crate-type
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```
- `cdylib`: WASM + 네이티브 동적 라이브러리
- `rlib`: Rust 내부 의존성

### 플랫폼별 의존성 분리
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
svg2pdf, usvg, pdf-writer, subsetter, ttf-parser

[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys, js-sys, wasm-bindgen
```

iOS 빌드 시 `not(wasm32)` 경로를 사용하므로 svg2pdf, usvg 등이 포함됨.

## 구현 단계 (3단계)

### 1단계: iOS 타겟 설치 + 크로스 컴파일 확인

**작업 내용:**
1. `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
2. `cargo build --target aarch64-apple-ios --lib` 실행
3. 컴파일 오류 확인 및 해결
4. `crate-type`에 `"staticlib"` 추가 — iOS에서 정적 라이브러리(.a) 필요

**대상 파일:** `Cargo.toml`

**수정 내용:**
```toml
[lib]
crate-type = ["cdylib", "rlib", "staticlib"]
```

**검증:** `target/aarch64-apple-ios/release/librhwp.a` 생성 확인

---

### 2단계: C-ABI FFI 인터페이스

**대상 파일:** `src/ios_ffi.rs` (신규), `src/lib.rs`

**작업 내용:**
1. 최소 FFI 함수 정의:
```rust
#[no_mangle]
pub extern "C" fn rhwp_open(data: *const u8, len: usize) -> *mut HwpHandle;

#[no_mangle]
pub extern "C" fn rhwp_page_count(handle: *const HwpHandle) -> u32;

#[no_mangle]
pub extern "C" fn rhwp_render_page_svg(handle: *const HwpHandle, page: u32) -> *mut c_char;

#[no_mangle]
pub extern "C" fn rhwp_free_string(ptr: *mut c_char);

#[no_mangle]
pub extern "C" fn rhwp_close(handle: *mut HwpHandle);
```
2. `#[cfg(target_os = "ios")]` 또는 `#[cfg(not(target_arch = "wasm32"))]`로 조건부 컴파일
3. cbindgen으로 C 헤더(`rhwp.h`) 자동 생성

**검증:** cbindgen 실행 → `rhwp.h` 생성 확인

---

### 3단계: Xcode 프로젝트 연동 테스트

**대상:** `rhwp-ios/` (신규 Xcode 프로젝트)

**작업 내용:**
1. Xcode에서 iOS 앱 프로젝트 생성 (SwiftUI)
2. `librhwp.a` + `rhwp.h`를 프로젝트에 링크
3. Swift에서 Rust 함수 호출:
```swift
let data = try Data(contentsOf: hwpFileURL)
let handle = data.withUnsafeBytes { ptr in
    rhwp_open(ptr.baseAddress, data.count)
}
let pageCount = rhwp_page_count(handle)
print("페이지 수: \(pageCount)")
rhwp_close(handle)
```
4. iOS Simulator + 실제 기기에서 테스트

**검증:** Swift에서 HWP 파일 로드 → 페이지 수 출력

## 리스크

| 리스크 | 대응 |
|--------|------|
| WASM 전용 코드가 iOS 빌드에 포함 | `#[cfg]` 조건부 컴파일로 분리 |
| svg2pdf/usvg iOS 미지원 | iOS 빌드에서 제외 (`cfg` 분기) |
| staticlib 추가 시 기존 빌드 영향 | WASM, 네이티브 빌드 회귀 테스트 |
| cbindgen 미설치 | `cargo install cbindgen` |
