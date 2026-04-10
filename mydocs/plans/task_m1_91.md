# Task #91 — 수행계획서

## Swift UI + Rust FFI 브릿지 고도화

### 배경

Task #90에서 최소 FFI 5개 함수(open/page_count/render_page_svg/free_string/close)를 구현하여 iPad Simulator에서 동작을 검증했다. 현재 한계:

- 문서를 열면 핸들이 `loadSampleHWP()` 로컬 스코프에서 즉시 해제됨 → 페이지 이동 불가
- 에러 정보가 null/0으로만 전달됨 → Swift 측 디버깅 어려움
- 문서 메타데이터(제목, 생성일 등) 조회 API 없음
- 페이지 크기 정보 없음 → 뷰어 레이아웃 최적화 불가

### 목표

1. **핸들 수명 관리**: Swift 클래스로 래핑하여 문서 열림 동안 핸들 유지
2. **FFI API 확장**: 메타데이터 조회, 페이지 크기, 에러 메시지 전달
3. **Swift 래퍼 클래스**: `RhwpDocument` Swift 클래스로 안전한 FFI 호출 캡슐화

### 범위

- Rust 측: `ios_ffi.rs` FFI 함수 추가
- Swift 측: `RhwpDocument.swift` 래퍼 클래스 신규 작성
- C 헤더: `rhwp.h` 갱신
- ContentView: 래퍼 클래스 사용으로 리팩터

### 범위 외

- 다중 페이지 UI (#92에서 처리)
- Core Graphics 네이티브 렌더러 (#93)
- 파일 선택 UI (#92에서 처리)

### 위험 요소

- FFI 메모리 관리 오류 (double free, use-after-free)
- Rust 패닉이 C-ABI 경계를 넘으면 UB 발생 → catch_unwind 필요

### 산출물

| 파일 | 내용 |
|------|------|
| `src/ios_ffi.rs` | FFI 함수 추가 (메타데이터, 페이지 크기, 에러) |
| `rhwp-ios/Sources/rhwp.h` | C 헤더 갱신 |
| `rhwp-ios/Sources/RhwpDocument.swift` | Swift 래퍼 클래스 |
| `rhwp-ios/Sources/ContentView.swift` | 래퍼 클래스 적용 |
| `mydocs/working/task_m1_91_stage*.md` | 단계별 완료보고서 |
| `mydocs/report/task_m1_91_report.md` | 최종 완료보고서 |
