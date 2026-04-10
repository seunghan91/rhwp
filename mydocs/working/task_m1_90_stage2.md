# Task #90 — 2단계 완료보고서

## C-ABI FFI 인터페이스 ✅

### 생성 파일

| 파일 | 내용 |
|------|------|
| `src/ios_ffi.rs` | Rust FFI 함수 5개 (C-ABI) |
| `cbindgen.toml` | cbindgen 설정 |
| `rhwp-ios/rhwp.h` | 자동 생성 C 헤더 |

### FFI API

| 함수 | 용도 |
|------|------|
| `rhwp_open(data, len)` | HWP 바이트 데이터 → 핸들 |
| `rhwp_page_count(handle)` | 페이지 수 반환 |
| `rhwp_render_page_svg(handle, page)` | 페이지 SVG 렌더링 |
| `rhwp_free_string(ptr)` | SVG 문자열 해제 |
| `rhwp_close(handle)` | 핸들 해제 |

### 조건부 컴파일

- `#[cfg(not(target_arch = "wasm32"))]` — WASM 빌드에서 제외
- `lib.rs`에 `pub mod ios_ffi` 등록

### 검증

- `cargo build --target aarch64-apple-ios --lib --release`: ✅ 성공
- `cbindgen` → `rhwp-ios/rhwp.h`: ✅ 5개 함수 포함
- `cargo test`: 785 passed, 0 failed
