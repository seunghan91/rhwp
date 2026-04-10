# Task #90 — 3단계 완료보고서

## Xcode 프로젝트 연동 + iPad Simulator 테스트 ✅

### 작업 내용

1. **xcodegen으로 Xcode 프로젝트 생성** (`rhwp-ios/AlHangeul.xcodeproj`)
2. **SwiftUI 앱 구현** — ContentView + SVGWebView (WKWebView)
3. **Rust FFI 연동** — `librhwp.a` 링크 + Bridging Header
4. **iPad Simulator에서 실행** — HWP 파일 로드 + SVG 렌더링 성공

### 검증 결과

| 항목 | 결과 |
|------|------|
| Rust FFI 호출 (rhwp_open) | ✅ |
| 페이지 수 조회 (rhwp_page_count) | ✅ 상단 바에 표시 |
| SVG 렌더링 (rhwp_render_page_svg) | ✅ WKWebView에 표시 |
| iPad Simulator 실행 | ✅ |

### 프로젝트 구조

```
rhwp-ios/
├── AlHangeul.xcodeproj     — xcodegen 생성
├── project.yml             — xcodegen 스펙
├── rhwp.h                  — cbindgen 생성 (전체)
├── Sources/
│   ├── AlHangeulApp.swift  — SwiftUI App 엔트리
│   ├── ContentView.swift   — 뷰어 UI + Rust FFI 호출
│   ├── Info.plist          — 앱 메타데이터
│   ├── rhwp.h              — 최소 FFI 헤더
│   └── rhwp-Bridging-Header.h
└── Resources/
    └── sample.hwpx         — 테스트용 샘플
```

### 해결한 문제

| 문제 | 원인 | 해결 |
|------|------|------|
| Bridging Header 오류 | cbindgen 전체 헤더에 불필요 상수 | 최소 FFI 헤더 직접 작성 |
| 스킴명 한글 빌드 오류 | 빌드 경로에 한글 포함 | 타겟명 영문(AlHangeul), PRODUCT_NAME만 한글 |
| 링크 symbol not found | staticlib 빌드 미갱신 | cargo build 재실행 |
| 리소스 미포함 | xcodegen resources 설정 | 앱 번들에 수동 복사 (빌드 자동화 보류) |
