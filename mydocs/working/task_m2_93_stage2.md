# Task #93 — 2단계 완료보고서

## Swift 렌더 트리 Codable 모델 ✅

### 작업 내용

`rhwp-ios/Sources/RenderTree.swift` 신규 작성 — Rust 렌더 트리의 serde JSON에 대응하는 Swift Decodable 타입 전체.

### 구현된 타입 (약 30개)

| 카테고리 | 타입 |
|----------|------|
| **트리 구조** | RenderNode, RenderNodeType (21 variant), BBox |
| **노드 데이터** | PageNode, PageBackgroundNode, BodyNode, TextLineNode, TextRunNode, TableNode, TableCellNode, LineNode, RectangleNode, EllipseNode, PathNode, ImageNode, GroupNode, EquationNode, FormObjectNode, FootnoteMarkerNode |
| **스타일** | TextStyle (34필드), ShapeStyle, LineStyle, ShapeTransform, GradientFillInfo, PatternFillInfo, ShadowStyleInfo, TabStopInfo, TabLeaderInfo |
| **Enum** | PathCommand, FieldMarkerType |
| **보조** | CharOverlapInfo, CellContext, CellPathEntry, DynamicKey, ArcToValue |

### serde externally tagged enum 디코딩

Rust serde의 기본 직렬화 방식:
- Unit variant: `"MasterPage"` → String 디코딩
- Newtype variant: `{"TextRun": {...}}` → DynamicKey 기반 keyed container
- Struct variant: `{"Body": {"clip_rect": ...}}` → 동일

Swift의 `init(from decoder:)`에서 커스텀 디코딩으로 처리.

### 검증 결과

- Xcode 빌드 (iPad Simulator): ✅ BUILD SUCCEEDED
