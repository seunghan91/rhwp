# Task #91 — 2단계 완료보고서

## Swift RhwpDocument + DocumentViewModel ✅

### 작업 내용

1. **`RhwpDocument.swift`** — Rust FFI 래퍼 (데이터 모델)
2. **`DocumentViewModel.swift`** — 뷰 모델 (ObservableObject)

### RhwpDocument (데이터 모델)

| 멤버 | 용도 |
|------|------|
| `init(data: Data) throws` | `rhwp_open` 호출, 실패 시 `RhwpError` throw |
| `deinit` | `rhwp_close` 호출 (자동 해제) |
| `pageCount: Int` | `rhwp_page_count` 래핑 |
| `pageSize(at:)` | `rhwp_page_size` → `(width, height)` 튜플 |
| `renderPageSVG(at:)` | `rhwp_render_page_svg` → `String?` (해제 포함) |

- `@MainActor` 제한으로 단일 스레드 접근 보장
- `OpaquePointer`로 불완전 C 구조체 핸들 보관

### DocumentViewModel (뷰 모델)

| 멤버 | 용도 |
|------|------|
| `document: RhwpDocument?` | 핸들 수명 관리 |
| `currentPage: Int` | 현재 페이지 (Published) |
| `svgContent: String` | 렌더링 결과 (Published) |
| `errorMessage: String?` | 에러 표시 (Published) |
| `loadDocument(data:)` | 문서 로드 |
| `loadSampleFromBundle()` | 번들 샘플 로드 |
| `renderCurrentPage()` | 현재 페이지 SVG 렌더링 |

- `@StateObject`로 ContentView에서 사용 예정 (3단계)
- `currentPage`로 #92 다중 페이지 확장 대비

### 해결한 문제

| 문제 | 원인 | 해결 |
|------|------|------|
| `RhwpHandle` not found in scope | 불완전 C 구조체는 Swift에서 타입 사용 불가 | `OpaquePointer`로 핸들 보관 |

### 검증 결과

- Xcode 빌드 (iPad Simulator): ✅ BUILD SUCCEEDED
- 기존 ContentView 동작 유지 (3단계에서 리팩터 예정)
