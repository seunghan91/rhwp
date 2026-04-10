# Task #93 — 수행계획서

## Core Graphics 네이티브 렌더러 + 파일 선택 + 다중 페이지

### 배경

M1에서 Rust FFI 파이프라인(#90)과 Swift 래퍼(#91)를 구축했다. 현재 렌더링 경로:

```
Rust 렌더 트리 → SVG 문자열 → WKWebView HTML 로드
```

이 경로는 프로세스 간 통신, HTML/SVG 파싱, WebKit 렌더링 파이프라인을 거치므로 오버헤드가 크다.

### 목표

1. **Core Graphics 네이티브 렌더러** — Rust 렌더 트리를 Core Graphics 드로잉 커맨드로 직접 변환
2. **파일 선택 UI** — UIDocumentPickerViewController로 HWP/HWPX 파일 열기 (#92 통합)
3. **다중 페이지 탐색** — 스크롤 기반 페이지 이동 (#92 통합)

목표 렌더링 경로:

```
Rust 렌더 트리 → JSON 커맨드 배열 → Swift Core Graphics 직접 그리기
```

### 기술 분석

#### 렌더 트리 구조 (기존)

렌더 트리는 **출력 포맷에 무관한 추상 표현**이다. Renderer trait을 구현하면 새로운 출력 포맷을 추가할 수 있다.

노드 타입:
- **텍스트**: TextLine, TextRun (text, TextStyle: 폰트/색상/굵기/밑줄)
- **도형**: Rectangle, Ellipse, Line, Path (ShapeStyle: 채우기/테두리/그라데이션)
- **표**: Table, TableCell (row/col span, text_direction)
- **미디어**: Image (bin_data, crop, fill_mode), Equation (SVG 조각)
- **구조**: Page, Body, Column, Header, Footer, TextBox, Group

좌표: 픽셀 절대좌표, 왼쪽상단 원점 (Core Graphics와 Y축 방향만 다름)

#### FFI 전달 방식 선택지

| 방식 | 장점 | 단점 |
|------|------|------|
| A. JSON 커맨드 배열 | 디버깅 용이, Swift Codable 디코딩 | 직렬화/역직렬화 오버헤드, 문자열 메모리 |
| B. C 구조체 배열 | 오버헤드 최소 | 복잡한 데이터(문자열, 가변길이) 표현 어려움 |
| C. Rust에서 직접 CG 호출 | 중간 변환 없음 | core-graphics crate iOS 지원 미확인, FFI 복잡도 ↑ |

**권장: A (JSON 커맨드 배열)** — M2 단계에서는 정확성 우선, 성능 최적화는 M3 이후 검토.

### 범위

**포함:**
- Rust 측 렌더 커맨드 JSON 직렬화 FFI
- Swift Core Graphics 렌더러 (UIView + draw)
- 파일 선택 UI (UIDocumentPickerViewController)
- 다중 페이지 스크롤

**제외:**
- Metal 가속 (M3 이후)
- Apple Pencil 어노테이션 (#94)
- 수식 렌더링 (기존 SVG 조각을 이미지로 변환하여 표시)
- 폰트 서브셋 임베딩 (iOS 시스템 폰트 사용)

### 위험 요소

| 위험 | 대응 |
|------|------|
| Core Graphics Y축 반전 (좌하단 원점) | CGContext.translateBy + scaleBy로 좌표계 변환 |
| 한글 폰트 가용성 | iOS 시스템 폰트 폴백 (Apple SD Gothic Neo 등) |
| JSON 직렬화 성능 | 페이지당 1회, 렌더 커맨드 수 모니터링 |
| 대용량 이미지 처리 | 바이너리 데이터는 별도 FFI로 전달, JSON에는 ID만 포함 |

### 산출물

| 파일 | 내용 |
|------|------|
| `src/ios_ffi.rs` | 렌더 커맨드 JSON 반환 FFI |
| `src/renderer/cg_commands.rs` | 렌더 트리 → CG 커맨드 변환 |
| `rhwp-ios/Sources/CGRenderer.swift` | Core Graphics 렌더러 |
| `rhwp-ios/Sources/DocumentView.swift` | CG 기반 렌더링으로 교체 |
| `rhwp-ios/Sources/DocumentPickerView.swift` | 파일 선택 UI |
| `rhwp-ios/Sources/ContentView.swift` | 파일 선택 + 다중 페이지 통합 |
