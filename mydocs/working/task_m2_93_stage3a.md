# Task #93 — 3a단계 완료보고서

## Core Graphics 렌더러 — 도형 + 이미지 ✅

### 작업 내용

1. **`CGTreeRenderer.swift`** — 렌더 트리 순회 + Core Graphics 그리기
2. **`PageCanvasView.swift`** — UIView 서브클래스 + SwiftUI UIViewRepresentable 래퍼
3. **`RhwpDocument.swift`** — `renderPageTree`, `imageData` 메서드 추가

### CGTreeRenderer 구현 범위

| 노드 타입 | 처리 |
|-----------|------|
| Page | 흰색 배경 + 자식 순회 |
| PageBackground | 배경색, 테두리, 그라데이션 |
| Body | clip_rect 기반 클리핑 |
| Rectangle | 둥근 모서리, fill/stroke, 그라데이션 |
| Line | 색상, 두께, 점선 |
| Ellipse | fill/stroke, 그라데이션 |
| Path | MoveTo/LineTo/CurveTo/ClosePath → CGPath |
| Image | bin_data_id → CGImage 캐시 → draw |
| Table/TableCell | 자식 순회, 셀 클리핑 |
| Group | saveGState + 자식 순회 + restoreGState |
| TextRun | **플레이스홀더** (기본 폰트, 3b단계에서 Core Text 교체) |

### 스타일 처리

| 스타일 | 처리 |
|--------|------|
| ShapeStyle.fill_color | `setFillColor` + `fillPath` |
| ShapeStyle.stroke_color/width | `setStrokeColor` + `setLineWidth` + `strokePath` |
| ShapeStyle.stroke_dash | `setLineDash` (Solid/Dash/Dot/DashDot/DashDotDot) |
| ShapeStyle.opacity | `setAlpha` |
| ShapeTransform | rotation + horz/vert flip (중심점 기준) |
| GradientFillInfo | 선형(`drawLinearGradient`) / 원형(`drawRadialGradient`) |
| Y축 변환 | `translateBy(y: pageHeight)` + `scaleBy(y: -1)` |

### 미구현 (후속 단계)

| 항목 | 단계 |
|------|------|
| 텍스트 Core Text 렌더링 | 3b |
| 폰트 폴백 매핑 | 3b |
| SVG arc → bezier 변환 | M3 (현재 직선 근사) |
| 패턴 채우기 정확 구현 | M3 |
| 수식 네이티브 렌더링 | M3 |

### 검증 결과

- Xcode 빌드 (iPad Simulator): ✅ BUILD SUCCEEDED
