# Task #91 — 3단계 완료보고서

## ContentView 리팩터 + 탭 확장 대비 구조 분리 ✅

### 작업 내용

1. **`DocumentView.swift` 신규** — 단일 문서 렌더링 뷰 (SVGWebView 포함)
2. **`ContentView.swift` 리팩터** — 앱 전체 구조 담당, DocumentViewModel → DocumentView 전달
3. **뷰 계층 분리** — 향후 탭 확장 대비

### 리팩터 전후 비교

**변경 전**:
```
ContentView (107줄)
├── 상단 바
├── SVGWebView (인라인)
├── loadSampleHWP() (직접 FFI 호출)
└── SVGWebView struct (인라인)
```

**변경 후**:
```
ContentView (13줄) — 앱 구조 + ViewModel 생성
└── DocumentView — 단일 문서 렌더링
    ├── 상단 정보 바 (페이지 수 + 크기)
    ├── 로딩 상태 (ProgressView)
    ├── SVG 렌더링 (SVGWebView)
    └── 에러 표시

DocumentViewModel — 상태 관리
└── RhwpDocument — FFI 래퍼 (핸들 수명 관리)
```

### 탭 확장 시 구조

```swift
// 향후 탭 확장 시 ContentView만 수정
struct ContentView: View {
    var body: some View {
        TabView {
            DocumentView(viewModel: vm1)  // 파일 A
            DocumentView(viewModel: vm2)  // 파일 B
        }
    }
}
```

`DocumentView`와 `DocumentViewModel`은 수정 없이 재사용 가능.

### 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-ios/Sources/DocumentView.swift` | 신규 — 단일 문서 렌더링 뷰 + SVGWebView |
| `rhwp-ios/Sources/ContentView.swift` | 리팩터 — 13줄로 축소, ViewModel 연결만 담당 |

### 검증 결과

- Xcode 빌드 (iPad Simulator): ✅ BUILD SUCCEEDED
- iPad Simulator 실행 (iPad Pro 11-inch M4): ✅ 정상 동작
  - "알한글" 타이틀 표시 ✅
  - "1/66쪽" 페이지 수 표시 ✅ (`rhwp_page_count` → `DocumentViewModel.pageCount`)
  - "(793×1122pt)" 페이지 크기 표시 ✅ (`rhwp_page_size` → `DocumentViewModel.currentPageSize`)
  - SVG 렌더링 정상 ✅ (`rhwp_render_page_svg` → `DocumentView.SVGWebView`)
  - 크래시/에러 로그 없음 ✅
- `cargo test`: 785 passed, 0 failed (회귀 없음)
