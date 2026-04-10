# Task #91 — 구현계획서 (v2, 피드백 반영)

## Swift UI + Rust FFI 브릿지 고도화

### 피드백 반영 사항

리뷰 피드백(`mydocs/feedback/task_m1_91_impl_review.md`) 반영:

| 피드백 | 반영 |
|--------|------|
| `last_error` / `rhwp_error_message` M1 제외 | ✅ 제외. M2에서 Mutex 기반 도입 |
| `rhwp_document_info` M1 제외 | ✅ 제외. 페이지 수는 기존 `rhwp_page_count`로 충분 |
| `rhwp_page_info` C 구조체 반환 | ✅ `RhwpPageSize` C 구조체로 변경 |
| `catch_unwind` 선별 적용 + 매크로 | ✅ `ffi_guard!` 매크로, 패닉 가능 함수에만 적용 |
| RhwpDocument / DocumentViewModel 분리 | ✅ 데이터 모델 / 뷰 모델 분리 |
| `init throws` 사용 | ✅ `RhwpError` enum 정의 |
| `@MainActor` 제한 | ✅ Sendable은 M3 이후 검토 |
| SVG 캐시 → #92 이관 | ✅ 명시 |

### 기존 API 현황

| 기존 메서드 | 반환 | 비고 |
|-------------|------|------|
| `get_page_info(page_num)` | JSON (width, height, 여백) | 픽셀 단위 |
| `render_page_svg_native(page_num)` | `Result<String, HwpError>` | SVG 문자열 |

### 구현 단계 (3단계)

---

#### 1단계: Rust FFI 함수 확장

`src/ios_ffi.rs` 변경:

**1-1. `ffi_guard!` 매크로 도입**

```rust
macro_rules! ffi_guard {
    ($handle:expr, $default:expr, $body:expr) => {{
        if $handle.is_null() { return $default; }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(_) => $default,
        }
    }};
}
```

- `rhwp_open`, `rhwp_render_page_svg`, `rhwp_page_size`에 적용 (패닉 가능)
- `rhwp_page_count`, `rhwp_free_string`, `rhwp_close`는 null 체크만 (패닉 가능성 극히 낮음)

**1-2. FFI 함수 추가: 페이지 크기**

```rust
#[repr(C)]
pub struct RhwpPageSize {
    pub width_pt: f64,
    pub height_pt: f64,
}

#[no_mangle]
pub extern "C" fn rhwp_page_size(handle: *const RhwpHandle, page: u32) -> RhwpPageSize;
```

- JSON 파싱 오버헤드 없이 C 구조체로 직접 반환
- 실패 시 `RhwpPageSize { width_pt: 0.0, height_pt: 0.0 }` 반환

**1-3. 기존 함수에 `ffi_guard!` 매크로 적용**

`rhwp_open`, `rhwp_render_page_svg`에 catch_unwind 적용.

**1-4. C 헤더 갱신** (`rhwp-ios/Sources/rhwp.h`)

**검증**: `cargo build --target aarch64-apple-ios-sim --lib --release` 성공 + `cargo test` 통과

---

#### 2단계: Swift RhwpDocument + DocumentViewModel

**2-1. `RhwpDocument.swift` — FFI 래퍼 (데이터 모델)**

```swift
@MainActor
class RhwpDocument {
    private let handle: OpaquePointer

    init(data: Data) throws { ... }  // rhwp_open, 실패 시 RhwpError.parseFailure throw
    deinit { rhwp_close(handle) }

    var pageCount: Int { ... }
    func pageSize(at page: Int) -> RhwpPageSize { ... }
    func renderPageSVG(at page: Int) -> String? { ... }
}

enum RhwpError: LocalizedError {
    case parseFailure
    case invalidData
}
```

- `rhwp_open`이 데이터를 파싱 후 IR로 복사하므로 `withUnsafeBytes` 밖에서 핸들 사용 안전
- SVG 캐싱은 #92에서 구현

**2-2. `DocumentViewModel.swift` — 뷰 모델**

```swift
@MainActor
class DocumentViewModel: ObservableObject {
    @Published var document: RhwpDocument?
    @Published var currentPage: Int = 0
    @Published var svgContent: String = ""
    @Published var errorMessage: String?

    func loadDocument(data: Data) { ... }
    func renderCurrentPage() { ... }
}
```

- `@StateObject`로 ContentView에서 사용
- #92 다중 페이지에서 `currentPage` 상태 관리에 자연스럽게 확장

**검증**: Xcode 빌드 성공 + iPad Simulator에서 기존과 동일하게 동작

---

#### 3단계: ContentView 리팩터 + 통합 검증

- `@StateObject private var viewModel = DocumentViewModel()`
- 페이지 크기를 활용한 SVG 뷰포트 설정
- 에러 시 `viewModel.errorMessage` 표시
- 상단 바에 페이지 수 표시 (기존 기능 유지)

**검증**: iPad Simulator에서 전체 플로우 동작 확인 + `cargo test` 회귀 없음

---

### 파일 변경 목록

| 파일 | 변경 | 단계 |
|------|------|------|
| `src/ios_ffi.rs` | `ffi_guard!` 매크로, `RhwpPageSize` 구조체, `rhwp_page_size` 함수, 기존 함수 catch_unwind 적용 | 1 |
| `rhwp-ios/Sources/rhwp.h` | C 헤더 갱신 (`RhwpPageSize`, `rhwp_page_size`) | 1 |
| `rhwp-ios/Sources/RhwpDocument.swift` | FFI 래퍼 클래스 + RhwpError 신규 | 2 |
| `rhwp-ios/Sources/DocumentViewModel.swift` | 뷰 모델 신규 | 2 |
| `rhwp-ios/Sources/ContentView.swift` | ViewModel 적용 리팩터 | 3 |

### M1 범위에서 제외 (후속 마일스톤 이관)

| 항목 | 이관 대상 |
|------|-----------|
| `rhwp_error_message` + `last_error` 패턴 | M2 (Mutex 기반) |
| `rhwp_document_info` 메타데이터 | M2 (문서 정보 UI와 함께) |
| SVG 렌더링 캐시 | #92 (다중 페이지) |
| Sendable / 백그라운드 렌더링 | M3 (#93 네이티브 렌더러) |
