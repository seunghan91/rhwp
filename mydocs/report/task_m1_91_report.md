# Task #91 — 최종 완료보고서

## Swift UI + Rust FFI 브릿지 고도화

### 목표

Task #90의 최소 FFI 5개 함수를 고도화하여, 안전한 핸들 수명 관리 + 패닉 방어 + 페이지 크기 조회 + 탭 확장 대비 뷰 구조를 구축한다.

### 단계별 결과

| 단계 | 내용 | 결과 |
|------|------|------|
| 1 | Rust FFI 확장 | ✅ `ffi_guard!` 매크로, `RhwpPageSize` C 구조체, `rhwp_page_size` 함수 |
| 2 | Swift 래퍼 클래스 | ✅ `RhwpDocument` (데이터 모델) + `DocumentViewModel` (뷰 모델) |
| 3 | ContentView 리팩터 | ✅ `DocumentView` 분리, 탭 확장 대비 구조 |

### 피드백 반영 사항

리뷰 피드백(`mydocs/feedback/task_m1_91_impl_review.md`) 반영:

| 피드백 | 반영 |
|--------|------|
| `last_error` / `rhwp_error_message` M1 제외 | ✅ M2 이관 |
| `rhwp_document_info` M1 제외 | ✅ M2 이관 |
| `rhwp_page_info` → C 구조체 반환 | ✅ `RhwpPageSize` 구조체 |
| `catch_unwind` 선별 적용 + 매크로 | ✅ 패닉 가능 3개 함수에만 적용 |
| RhwpDocument / DocumentViewModel 분리 | ✅ 데이터/뷰 모델 분리 |
| `init throws` | ✅ `RhwpError` enum |
| `@MainActor` 제한 | ✅ |

### 생성/변경 파일

| 파일 | 역할 |
|------|------|
| `src/ios_ffi.rs` | `ffi_guard!` 매크로, `RhwpPageSize`, `rhwp_page_size`, catch_unwind 적용 |
| `rhwp-ios/Sources/rhwp.h` | C 헤더 갱신 (`RhwpPageSize`, `rhwp_page_size`) |
| `rhwp-ios/Sources/RhwpDocument.swift` | FFI 래퍼 클래스 (핸들 수명 관리) |
| `rhwp-ios/Sources/DocumentViewModel.swift` | 뷰 모델 (ObservableObject) |
| `rhwp-ios/Sources/DocumentView.swift` | 단일 문서 렌더링 뷰 |
| `rhwp-ios/Sources/ContentView.swift` | 앱 전체 구조 (13줄) |

### FFI API (최종)

```c
// 기존 (Task #90)
RhwpHandle *rhwp_open(const uint8_t *data, size_t len);
uint32_t rhwp_page_count(const RhwpHandle *handle);
char *rhwp_render_page_svg(const RhwpHandle *handle, uint32_t page);
void rhwp_free_string(char *ptr);
void rhwp_close(RhwpHandle *handle);

// 신규 (Task #91)
RhwpPageSize rhwp_page_size(const RhwpHandle *handle, uint32_t page);
```

### 아키텍처

```
ContentView (앱 구조)
└── DocumentView (단일 문서 렌더링)
    └── @ObservedObject DocumentViewModel
        └── RhwpDocument (FFI 래퍼, init/deinit)
            └── Rust FFI (ffi_guard! 패닉 방어)
```

탭 확장 시 `ContentView`에 `TabView` 추가, `DocumentView`/`DocumentViewModel` 수정 없이 재사용.

### 검증 결과

- iPad Simulator (iPad Pro 11-inch M4): ✅ HWP 로드 + SVG 렌더링 + 페이지 크기 표시
- `cargo build --target aarch64-apple-ios-sim --lib --release`: ✅
- `cargo test`: 785 passed, 0 failed (회귀 없음)

### 후속 작업

- #92: iOS 최소 뷰어 앱 (파일 선택 + 다중 페이지)
- M2 이관: `rhwp_error_message`, `rhwp_document_info`, SVG 캐싱
