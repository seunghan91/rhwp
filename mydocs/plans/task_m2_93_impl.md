# Task #93 — 구현계획서 (v2, 피드백 반영)

## Core Graphics 네이티브 렌더러 + 파일 선택 + 다중 페이지

### 피드백 반영 사항

리뷰 피드백(`mydocs/feedback/task_m2_93_impl_review.md`) 반영:

| 피드백 | 반영 |
|--------|------|
| P0: CanvasCommand 스타일 정보 손실 | ✅ B안 채택 — 렌더 트리 직접 직렬화 (정보 손실 제로) |
| P1: 폰트 폴백 매핑 | ✅ 3b단계에 폰트 매핑 테이블 포함 |
| P1: 이미지 CGImage 캐시 | ✅ 3a단계에 이미지 캐시 포함 |
| P2: 3단계 분할 (도형/텍스트) | ✅ 3a(도형+이미지) / 3b(텍스트+폰트) 분리 |
| P2: 다중 페이지 메모리 관리 | ✅ onAppear/onDisappear 메모리 해제 |
| P3: 줌 미지원 명시 | ✅ M2 범위 외 명시 |
| serde cfg 적용 범위 | ✅ Serialize derive는 무조건, 직렬화 호출만 cfg |
| Core Text 사용 | ✅ NSAttributedString 대신 Core Text 기본 |
| 이미지 FFI 안전성 | ✅ Data(bytes:count:) 복사 방식 |

### 설계 개요

**B안: 렌더 트리(PageRenderTree) 직접 직렬화**

CanvasCommand는 테스트용으로 스타일 정보를 대부분 버린다. 대신 렌더 트리 자체를 serde로 직렬화하여 Swift에 전달한다. 정보 손실이 없으므로 고품질 렌더링이 가능하다.

```
Rust: build_page_tree() → PageRenderTree
  ↓ serde_json::to_string()
FFI: rhwp_render_page_tree(handle, page) → JSON 문자열
  ↓
Swift: JSONDecoder → RenderTree 모델
  ↓ 트리 순회
Core Graphics: CGContext API 직접 호출
```

Swift 측 렌더러가 렌더 트리의 모든 노드 타입과 스타일을 이해해야 하므로 복잡도가 높지만, **렌더링 품질과 정확성에서 최적**이다.

### 구현 단계 (6단계)

---

#### 1단계: Rust — serde Serialize 도입 + 렌더 트리 직렬화 FFI

**1-1. serde 의존성 추가**

```toml
# Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"   # 직렬화 호출은 cfg(not(wasm32))로 제한
```

`Serialize` derive는 모든 타겟에서 적용. `serde_json::to_string()` 호출만 iOS FFI에서 사용.

**1-2. 렌더 트리 관련 타입에 Serialize derive 추가**

대상 구조체/enum:
- `RenderNode`, `RenderNodeType`, `BoundingBox`
- `PageNode`, `TextRunNode`, `TextLineNode`, `TableNode`, `TableCellNode`
- `LineNode`, `RectangleNode`, `EllipseNode`, `PathNode`, `ImageNode`
- `GroupNode`, `EquationNode`, `FormObjectNode`, `FootnoteMarkerNode`
- `PageBackgroundNode`
- `TextStyle`, `ShapeStyle`, `LineStyle`, `ColorRef`, `StrokeDash`
- `GradientFillInfo`, `GradientStop`, `PatternFillInfo`
- `PathCommand`, `ShapeTransform`
- `ShadowStyle`, `ArrowStyle`, `LineRenderType`

ImageNode의 `data: Option<Vec<u8>>` 필드는 `#[serde(skip)]`으로 제외 (이미지는 별도 FFI).

**1-3. FFI 함수 추가**

```rust
// src/ios_ffi.rs
#[no_mangle]
pub extern "C" fn rhwp_render_page_tree(
    handle: *const RhwpHandle, page: u32
) -> *mut c_char;  // 렌더 트리 JSON 반환

#[no_mangle]
pub extern "C" fn rhwp_image_data(
    handle: *const RhwpHandle, bin_data_id: u16, out_len: *mut usize
) -> *const u8;  // 이미지 바이너리 참조 반환
```

**1-4. C 헤더 갱신**

**검증**: `cargo build --target aarch64-apple-ios-sim --lib --release` + `cargo test` + JSON 출력 스냅샷 확인

---

#### 2단계: Swift — 렌더 트리 Codable 모델

**2-1. `RenderTree.swift` — 렌더 트리 Swift 모델**

Rust의 RenderNode/RenderNodeType에 대응하는 Codable 구조체:

```swift
struct RenderNode: Decodable {
    let id: Int
    let nodeType: RenderNodeType
    let bbox: BoundingBox
    let children: [RenderNode]
    let visible: Bool
}

enum RenderNodeType: Decodable {
    case page(PageNode)
    case textRun(TextRunNode)
    case textLine(TextLineNode)
    case table(TableNode)
    case tableCell(TableCellNode)
    case rectangle(RectangleNode)
    case line(LineNode)
    case ellipse(EllipseNode)
    case path(PathNode)
    case image(ImageNode)
    case group(GroupNode)
    // 구조 노드: body, column, header, footer, masterPage, textBox 등
}
```

**2-2. 스타일 모델**

```swift
struct TextStyle: Decodable {
    let fontFamily: String
    let fontSize: Double
    let color: ColorRef
    let bold: Bool
    let italic: Bool
    let ratio: Double          // 장평
    let letterSpacing: Double  // 자간
    let underline: UInt8
    let strikethrough: Bool
    // ... 44개 필드 전체
}

struct ShapeStyle: Decodable { ... }
struct LineStyle: Decodable { ... }
```

**검증**: Xcode 빌드 성공 + 샘플 JSON 디코딩 테스트

---

#### 3a단계: Core Graphics 렌더러 — 도형 + 이미지

**3a-1. `CGTreeRenderer.swift` — 렌더 트리 순회 + 도형 그리기**

```swift
@MainActor
class CGTreeRenderer {
    private var imageCache: [UInt16: CGImage] = [:]

    func render(tree: RenderNode, in context: CGContext, document: RhwpDocument?)
    // 재귀 순회: renderNode(_ node:, in context:)
}
```

노드별 Core Graphics 매핑:
- `Rectangle` → `context.addRect()` / `context.addRoundedRect()` + fill/stroke
- `Line` → `context.move(to:)` + `context.addLine(to:)` + stroke (color, width, dash)
- `Ellipse` → `context.addEllipse(in:)` + fill/stroke
- `Path` → MoveTo/LineTo/CurveTo/ArcTo → `context.addPath()` (ArcTo → cubic bezier 변환)
- `Image` → `rhwp_image_data` FFI → `CGImage` 캐시 → `context.draw()`
- `Table`/`TableCell` → 자식 순회 + border_fill 렌더링
- `Group` → `context.saveGState()` + transform + 자식 순회 + `context.restoreGState()`

ShapeStyle 처리:
- `fill_color` → `context.setFillColor()`
- `stroke_color` + `stroke_width` → `context.setStrokeColor()` + `context.setLineWidth()`
- `stroke_dash` → `context.setLineDash()`
- `opacity` → `context.setAlpha()`
- `gradient` → `CGGradient` + `context.drawLinearGradient()` / `context.drawRadialGradient()`
- `pattern` → 패턴 이미지 생성 + `context.setFillColor(patternSpace:)`
- `shadow` → `context.setShadow()`

**3a-2. Y축 변환**

```swift
context.translateBy(x: 0, y: pageHeight)
context.scaleBy(x: 1, y: -1)
```

**3a-3. `PageCanvasView.swift` — UIView 서브클래스**

```swift
class PageCanvasView: UIView {
    var renderTree: RenderNode?
    var renderer: CGTreeRenderer
    override func draw(_ rect: CGRect) { ... }
}
```

`layer.contentsScale = UIScreen.main.scale` 설정 (Retina 대응).

**검증**: 도형/이미지/표가 포함된 페이지의 시각적 렌더링 확인 (텍스트는 아직 기본 표시)

---

#### 3b단계: Core Graphics 렌더러 — 텍스트 + 폰트 폴백

**3b-1. Core Text 기반 텍스트 렌더링**

TextRunNode 처리:
- `CTFontCreateWithName()` + `CTFontCreateCopyWithAttributes()` (장평 적용)
- `kCTKernAttributeName` (자간)
- `kCTForegroundColorAttributeName` (색상)
- `CTLineDraw()` 또는 `CTRunDraw()` (glyph 단위)

텍스트 효과:
- 밑줄/취소선: `context.move(to:)` + `context.addLine(to:)` (유형별 선 스타일)
- 강조점: 글자 위에 점 찍기
- 그림자: `context.setShadow()`
- 윤곽/양각/음각: stroke + fill 조합
- 위/아래 첨자: fontSize 축소 + y 오프셋
- 음영(형광펜): 텍스트 배경에 색상 사각형

**3b-2. 폰트 폴백 매핑 테이블**

`mydocs/tech/font_fallback_strategy.md` 참조하여 매핑:

| HWP 폰트 | iOS 폴백 |
|-----------|----------|
| 한컴돋움/함초롬돋움/HY중고딕/굴림/돋움 | Apple SD Gothic Neo |
| 한컴바탕/함초롬바탕/HY신명조/궁서/바탕 | AppleMyungjo |
| 한컴고딕 | Apple SD Gothic Neo Bold |
| Arial/Calibri/Tahoma/Verdana | Helvetica Neue |
| Times New Roman | Times New Roman |
| Courier New | Courier New |

**3b-3. Y축 텍스트 반전 처리**

Core Text는 자체 Y축 관리를 하므로, 텍스트 그리기 전 좌표 변환 프로토타이핑:

```swift
// 텍스트 그리기 전 좌표 변환
context.saveGState()
context.translateBy(x: x, y: y + fontSize)
context.scaleBy(x: 1, y: -1)
// CTLineDraw(...)
context.restoreGState()
```

**검증**: 다양한 글꼴/크기/스타일의 텍스트가 올바르게 렌더링되는지 시각 비교 (SVG 출력과 대조)

---

#### 4단계: DocumentView 교체 (WKWebView → Core Graphics)

**4-1. RhwpDocument에 renderPageTree 메서드 추가**

```swift
func renderPageTree(at page: Int) -> RenderNode?
```

`rhwp_render_page_tree` FFI 호출 → JSON → Decodable 디코딩.

**4-2. DocumentViewModel 수정**

- `svgContent: String` → `pageTree: RenderNode?`
- `renderCurrentPage()`에서 SVG 대신 렌더 트리 생성

**4-3. DocumentView 수정**

- `SVGWebView` → `PageCanvasView` (UIViewRepresentable)
- 페이지 크기 기반 뷰 사이즈 설정

**검증**: iPad Simulator에서 기존 샘플 파일 렌더링 확인 — SVG 출력과 시각 비교

---

#### 5단계: 파일 선택 + 다중 페이지 + 통합 검증

**5-1. `DocumentPickerView.swift` — 파일 선택 UI**

UIDocumentPickerViewController 래핑. UTType 등록 (com.hancom.hwp, com.hancom.hwpx, public.data).

**5-2. ContentView에 파일 선택 버튼 추가**

- 툴바에 "열기" 버튼 → DocumentPickerView 표시
- 선택한 파일 → `viewModel.loadDocument(data:)` 호출

**5-3. 다중 페이지 스크롤 + 메모리 관리**

```swift
ScrollView {
    LazyVStack(spacing: 8) {
        ForEach(0..<pageCount, id: \.self) { index in
            PageCanvasView(...)
                .frame(width: pageWidth, height: pageHeight)
                .onAppear { viewModel.loadPage(index) }
                .onDisappear { viewModel.unloadPage(index) }
        }
    }
}
```

- `loadPage`: 렌더 트리 생성 + 렌더링
- `unloadPage`: 렌더 트리/이미지 캐시 해제 (메모리 보호)

**5-4. 현재 페이지 번호 표시**

스크롤 위치 기반으로 현재 보이는 페이지 번호 갱신.

**검증**: iPad Simulator에서 전체 플로우 (파일 열기 → 렌더링 → 페이지 스크롤) 동작 확인 + `cargo test` 회귀 없음

---

### 파일 변경 목록

| 파일 | 변경 | 단계 |
|------|------|------|
| `Cargo.toml` | serde + serde_json 의존성 추가 | 1 |
| `src/renderer/render_tree.rs` | RenderNode 등 Serialize derive | 1 |
| `src/renderer/mod.rs` | TextStyle/ShapeStyle/LineStyle 등 Serialize derive | 1 |
| `src/ios_ffi.rs` | `rhwp_render_page_tree`, `rhwp_image_data` FFI | 1 |
| `rhwp-ios/Sources/rhwp.h` | C 헤더 갱신 | 1 |
| `rhwp-ios/Sources/RenderTree.swift` | 렌더 트리 Codable 모델 | 2 |
| `rhwp-ios/Sources/CGTreeRenderer.swift` | CG 렌더러 — 도형 + 이미지 | 3a |
| `rhwp-ios/Sources/PageCanvasView.swift` | UIView 서브클래스 | 3a |
| `rhwp-ios/Sources/CGTreeRenderer.swift` | CG 렌더러 — 텍스트 + 폰트 | 3b |
| `rhwp-ios/Sources/FontFallback.swift` | 폰트 폴백 매핑 테이블 | 3b |
| `rhwp-ios/Sources/RhwpDocument.swift` | renderPageTree 메서드 추가 | 4 |
| `rhwp-ios/Sources/DocumentViewModel.swift` | SVG → 렌더 트리 전환 | 4 |
| `rhwp-ios/Sources/DocumentView.swift` | WKWebView → PageCanvasView 교체 | 4 |
| `rhwp-ios/Sources/DocumentPickerView.swift` | 파일 선택 UI 신규 | 5 |
| `rhwp-ios/Sources/ContentView.swift` | 파일 선택 + 다중 페이지 통합 | 5 |

### M2 범위에서 제외 (후속 이관)

| 항목 | 이관 대상 | 비고 |
|------|-----------|------|
| Metal 가속 렌더링 | M3 | |
| 수식 네이티브 렌더링 | M3 | 현재 SVG 조각 → 이미지 변환으로 표시 |
| 줌/핀치 지원 | M3 | M2는 1:1 보기만 |
| 접근성 (VoiceOver) | M3 | CG 직접 그리기는 접근성 트리 자동 생성 안됨 |
| 다크 모드 최적화 | M3 | |
| JSON → 바이너리 프로토콜 최적화 | M3 | M2는 JSON으로 충분 |
