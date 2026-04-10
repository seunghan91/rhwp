# Task #91 — 1단계 완료보고서

## Rust FFI 함수 확장 ✅

### 작업 내용

1. **`ffi_guard!` 매크로 도입** — null 핸들 체크 + `catch_unwind`로 패닉 방어
2. **`RhwpPageSize` C 구조체 추가** — `#[repr(C)]` + `Copy`/`Clone` derive
3. **`rhwp_page_size` 함수 추가** — 페이지 크기를 C 구조체로 반환 (JSON 파싱 오버헤드 없음)
4. **기존 FFI 함수 catch_unwind 적용** — `rhwp_open`, `rhwp_render_page_svg`에 패닉 방어 적용
5. **C 헤더 갱신** — `RhwpPageSize` typedef + `rhwp_page_size` 선언 추가

### `ffi_guard!` 적용 범위

| 함수 | catch_unwind | 사유 |
|------|:---:|------|
| `rhwp_open` | ✅ | 파싱 중 패닉 가능 |
| `rhwp_render_page_svg` | ✅ | 렌더링 중 패닉 가능 |
| `rhwp_page_size` | ✅ | get_page_info_native 내부 패닉 가능 |
| `rhwp_page_count` | ❌ | 단순 정수 반환, null 체크만 |
| `rhwp_free_string` | ❌ | 단순 해제, null 체크만 |
| `rhwp_close` | ❌ | 단순 해제, null 체크만 |

### 변경 파일

| 파일 | 변경 |
|------|------|
| `src/ios_ffi.rs` | `ffi_guard!` 매크로, `RhwpPageSize`, `rhwp_page_size`, `extract_json_f64`, 기존 함수 catch_unwind 적용 |
| `rhwp-ios/Sources/rhwp.h` | `RhwpPageSize` typedef, `rhwp_page_size` 선언 추가 |

### 검증 결과

- `cargo build --target aarch64-apple-ios-sim --lib --release`: ✅ 성공
- `cargo test`: 785 passed, 0 failed (회귀 없음)
