# Task #93 — 최종 완료보고서

## Core Graphics 네이티브 렌더러 + 파일 선택 + 다중 페이지

### 목표

WKWebView 기반 SVG 렌더링을 Core Graphics 네이티브 렌더링으로 전환하고, 파일 선택 + 다중 페이지 스크롤을 구현한다.

**변경 전**: Rust → SVG 문자열 → WKWebView HTML 로드
**변경 후**: Rust → 렌더 트리 JSON → Swift Decodable → Core Graphics 직접 그리기

### 단계별 결과

| 단계 | 내용 | 결과 |
|------|------|------|
| 1 | Rust serde + FFI | ✅ 35개 타입 Serialize, `rhwp_render_page_tree` + `rhwp_image_data` FFI |
| 2 | Swift Codable 모델 | ✅ RenderNode + 30개 타입 Decodable (serde externally tagged enum 대응) |
| 3a | CG 렌더러 — 도형 + 이미지 | ✅ rect/line/ellipse/path/image/gradient/pattern/shadow |
| 3b | CG 렌더러 — 텍스트 + 폰트 | ✅ Core Text + 장평/자간/밑줄/취소선 + 폰트 폴백 30+ 항목 |
| 4 | DocumentView 교체 | ✅ WKWebView → PageCanvasView, Y축 좌표계 해결 |
| 5 | 파일 선택 + 다중 페이지 | ✅ UIDocumentPicker + LazyVStack + 메모리 관리 |

### 피드백 반영

리뷰 피드백(`mydocs/feedback/task_m2_93_impl_review.md`) 반영:

| 피드백 | 반영 |
|--------|------|
| P0: CanvasCommand 스타일 손실 | ✅ B안 — 렌더 트리 직접 직렬화 (정보 손실 제로) |
| P1: 폰트 폴백 매핑 | ✅ FontFallback.swift 30+ 항목 |
| P1: 이미지 CGImage 캐시 | ✅ imageCache 딕셔너리 |
| P2: 3단계 분할 | ✅ 3a(도형) / 3b(텍스트) |
| P2: 다중 페이지 메모리 관리 | ✅ onAppear/onDisappear |
| Core Text 사용 | ✅ NSAttributedString 대신 Core Text |

### 생성/변경 파일

| 파일 | 역할 | 단계 |
|------|------|------|
| `Cargo.toml` | serde + serde_json 의존성 | 1 |
| `src/renderer/render_tree.rs` 외 9개 | Serialize derive (35개 타입) | 1 |
| `src/ios_ffi.rs` | `rhwp_render_page_tree`, `rhwp_image_data` FFI | 1 |
| `src/document_core/queries/rendering.rs` | `build_page_render_tree`, `get_bin_data` | 1 |
| `rhwp-ios/Sources/rhwp.h` | C 헤더 갱신 | 1 |
| `rhwp-ios/Sources/RenderTree.swift` | 렌더 트리 Codable 모델 (30개 타입) | 2 |
| `rhwp-ios/Sources/CGTreeRenderer.swift` | Core Graphics 렌더러 | 3a, 3b |
| `rhwp-ios/Sources/PageCanvasView.swift` | UIView + UIViewRepresentable | 3a |
| `rhwp-ios/Sources/FontFallback.swift` | HWP → iOS 폰트 매핑 | 3b |
| `rhwp-ios/Sources/RhwpDocument.swift` | renderPageTree, imageData 추가 | 4 |
| `rhwp-ios/Sources/DocumentViewModel.swift` | SVG → 렌더 트리, 다중 페이지 | 4, 5 |
| `rhwp-ios/Sources/DocumentView.swift` | WKWebView → PageCanvasView, 다중 페이지 | 4, 5 |
| `rhwp-ios/Sources/DocumentPickerView.swift` | 파일 선택 UI | 5 |
| `rhwp-ios/Sources/ContentView.swift` | 툴바 열기 버튼 | 5 |

### Y축 좌표계 해결

UIView.draw()의 CGContext는 UIKit이 이미 좌상단 원점으로 변환한 상태. 렌더 트리 좌표를 그대로 사용하고, Core Text와 CGImage만 bbox 영역 내에서 국소 Y반전.

### 검증 결과

- iPad Simulator (iPad Pro 11-inch M4): ✅ 66페이지 문서 스크롤 렌더링
- 텍스트 (한글/영문, 다양한 스타일): ✅
- 도형 (사각형, 직선, 타원, 패스): ✅
- 표 (행/열, 셀 내 텍스트): ✅
- 이미지 (JPG/PNG): ✅
- 파일 선택 (UIDocumentPicker): ✅
- `cargo test`: 785 passed, 0 failed (회귀 없음)

### 알려진 이슈

| 이슈 | 상태 | 비고 |
|------|------|------|
| 일부 BMP 이미지 미표시 | 미해결 | binDataId 매핑 또는 BMP 호환성 조사 필요 |
| SVG arc → bezier 변환 | 직선 근사 | M3에서 정확한 변환 |
| 패턴 채우기 | 배경색만 | M3에서 정확한 패턴 |
| 줌/핀치 | 미지원 | M3 |

### 후속 작업

- #94: Apple Pencil 어노테이션 레이어
- M3: Metal 가속, WMF/EMF 변환, 수식 네이티브, 줌, 접근성
- BMP 이미지 미표시 버그 수정
