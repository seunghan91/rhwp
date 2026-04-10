# Task #93 — 5단계 완료보고서

## 파일 선택 + 다중 페이지 스크롤 ✅

### 작업 내용

1. **`DocumentPickerView.swift`** — UIDocumentPickerViewController SwiftUI 래퍼
2. **`DocumentViewModel`** — 다중 페이지 지원 (pageTrees 캐시 + lazy loading)
3. **`DocumentView`** — ScrollView + LazyVStack 다중 페이지 스크롤
4. **`ContentView`** — 툴바 "열기" 버튼 + sheet 파일 선택

### 파일 선택

- UTType: HWP, HWPX, 일반 데이터
- 보안 범위 접근 (`startAccessingSecurityScopedResource`)
- 파일명 상단 바에 표시

### 다중 페이지

- `ScrollView` + `LazyVStack`: 66페이지 문서 스크롤 확인
- `onAppear`: 렌더 트리 lazy loading (화면에 진입할 때 생성)
- `onDisappear`: 렌더 트리 해제 (메모리 보호)
- 현재 페이지 번호 자동 갱신

### 알려진 제한사항

| 항목 | 상태 | 비고 |
|------|------|------|
| JPG/PNG 이미지 | ✅ 정상 | UIImage(data:) 디코딩 |
| WMF/EMF 이미지 | ❌ 미표시 | iOS에서 미지원 포맷, M3에서 변환 구현 |
| 줌/핀치 | 미지원 | M3 |

### 검증 결과

- iPad Simulator: ✅ 전체 플로우 동작 (샘플 로드 → 스크롤 → 66페이지)
- 텍스트/표/도형/이미지(JPG/PNG): ✅ Core Graphics 네이티브 렌더링
- `cargo test`: 785 passed, 0 failed (회귀 없음)
