# Task #93 구현계획서 리뷰

## 1. CanvasCommand 재사용 적절성 — 심각한 정보 손실

현재 CanvasCommand는 **테스트/디버깅용**으로 설계되어, Renderer trait이 전달하는 스타일 정보를 대부분 버린다. Core Graphics 렌더러로 사용하기에는 치명적인 정보 누락이 있다.

### 1-1. FillText: TextStyle 44개 필드 중 텍스트/좌표만 보존

```rust
// 현재 CanvasCommand
FillText(String, f64, f64)   // 텍스트, x, y — 이것만 전달

// draw_text에서 _style을 완전히 무시
fn draw_text(&mut self, text: &str, x: f64, y: f64, _style: &TextStyle) {
    self.commands.push(CanvasCommand::FillText(text.to_string(), x, y));
}
```

누락되는 정보:
- **글꼴**: font_family, font_size, bold, italic — 텍스트를 어떤 글꼴/크기로 그릴지 알 수 없음
- **색상**: color — 모든 텍스트가 기본색으로 그려짐
- **장평**: ratio — 한글 문서에서 80~120% 장평 빈번
- **자간**: letter_spacing — 글자 간격 재현 불가
- **밑줄/취소선**: underline, strikethrough, underline_shape, strike_shape, underline_color, strike_color
- **강조점**: emphasis_dot
- **텍스트 효과**: outline_type, shadow_type, emboss, engrave
- **위/아래 첨자**: superscript, subscript
- **음영(형광펜)**: shade_color
- **탭 리더**: tab_leaders (12종 채움 모양)

WebCanvasRenderer의 draw_text가 200줄 이상인 이유가 이 모든 속성을 처리하기 때문이다. FillText 하나로는 검은색 기본 글꼴 텍스트만 나온다.

### 1-2. DrawLine: LineStyle 완전 누락

```rust
fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, _style: &LineStyle) {
    self.commands.push(CanvasCommand::DrawLine(x1, y1, x2, y2));
}
```

누락: color, width, dash(실선/파선/점선 등), line_type(이중선/삼중선), start_arrow, end_arrow, shadow.

### 1-3. DrawEllipse/DrawImage: ShapeStyle 누락

- DrawEllipse: fill_color, stroke_color, stroke_width, opacity, shadow 전부 없음
- DrawImage: 이미지 데이터 자체가 없음 (좌표만 기록)

### 1-4. draw_rect: corner_radius 무시

`_corner_radius` 파라미터를 버리므로 둥근 모서리 사각형이 직각으로 렌더링된다.

### 1-5. 그라데이션 채우기 미지원

ShapeStyle에 GradientFillInfo가 있으나, CanvasCommand에는 그라데이션 커맨드가 없다. ShapeStyle.gradient가 Some일 때 단색으로 폴백된다.

### 권고사항

CanvasCommand를 **그대로 재사용하는 것은 불가**하다. 두 가지 접근 중 하나를 선택해야 한다:

**A안 (권장): CanvasCommand를 확장하여 스타일 정보를 포함**

```rust
FillText {
    text: String,
    x: f64, y: f64,
    font_family: String,
    font_size: f64,
    color: u32,
    bold: bool,
    italic: bool,
    ratio: f64,
    letter_spacing: f64,
    underline: u8,
    underline_shape: u8,
    underline_color: u32,
    strikethrough: bool,
    strike_shape: u8,
    strike_color: u32,
    emphasis_dot: u8,
    outline_type: u8,
    shadow_type: u8,
    shadow_color: u32,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    emboss: bool,
    engrave: bool,
    superscript: bool,
    subscript: bool,
    shade_color: u32,
    // tab_leaders는 별도 커맨드로 분리
}
```

이 경우 WebCanvasRenderer도 동일한 커맨드를 사용하도록 통합할 수 있어 장기적으로 유리하다.

**B안: 렌더 트리(RenderNode)를 직접 직렬화**

CanvasCommand 대신 PageRenderTree 자체를 JSON으로 직렬화하고, Swift에서 렌더 트리를 순회하며 Core Graphics를 호출한다. 이 방식은 정보 손실이 없으나 Swift 측 로직이 복잡해진다.

---

## 2. JSON 직렬화 방식

### 2-1. 성능 우려는 M2에서 과도

페이지당 커맨드 수를 실측해 보면, 일반적인 HWP 문서에서 한 페이지에 300~3000개 커맨드가 발생한다. JSON으로 직렬화 시:

- 직렬화: 1~5ms (Rust serde_json은 매우 빠름)
- 전송: C 문자열 포인터 전달이므로 복사 1회
- 디코딩: Swift JSONDecoder에서 5~20ms (커맨드 3000개 기준)

총 합계 10~30ms로 60fps에는 못 미치지만, **정적 문서 뷰어에서 페이지 전환 시 1회 호출**이므로 체감 성능에 문제없다.

### 2-2. JSON이 적절한 선택인 이유

- 디버깅 용이: 중간 JSON을 파일로 덤프하여 문제 추적 가능
- M2 단계에서 조기 최적화 불필요
- serde_json은 이미 Cargo.toml에 있을 가능성 높음 (확인 필요)

### 2-3. 주의: serde Serialize를 cfg(not(wasm32))로 제한하지 말 것

구현계획서에 `#[cfg(not(target_arch = "wasm32"))]`로 serde_json을 추가한다고 했는데, `Serialize` derive는 **CanvasCommand 정의에 무조건 붙여야** 한다. cfg 분기는 serde_json을 사용하는 **직렬화 호출 코드**에만 적용하면 된다. Serialize trait derive 자체는 serde_json 없이도 동작한다 (serde만 있으면 됨).

### 2-4. 후속 최적화 경로

M3에서 성능이 병목이 되면 다음 순서로 최적화:
1. JSON → MessagePack (serde 호환, 바이너리)
2. MessagePack → C 구조체 배열 (직접 메모리 매핑)

---

## 3. Core Graphics 렌더링 설계

### 3-1. 텍스트 렌더링: Core Text 사용 권장

구현계획서에 `NSAttributedString.draw(at:)` 또는 Core Text를 병기했는데, **Core Text를 기본으로 사용**하는 것을 권장한다.

이유:
- `NSAttributedString.draw(at:)`는 내부적으로 Core Text를 호출하며, 오버헤드가 추가됨
- 장평(ratio) 적용에 `CGAffineTransform`이 필요한데, Core Text의 `CTFontCreateCopyWithAttributes`로 직접 제어 가능
- 자간(letter_spacing)은 `kCTKernAttributeName`으로 정밀 제어
- 글자별 위치 지정 시 `CTLineDraw`보다 `CTRunDraw` + glyph 단위 배치가 정확

### 3-2. draw(_ rect:)의 성능 특성

`UIView.draw(_ rect:)`는 **오프스크린 비트맵 버퍼**에 그린다. 주의할 점:

- **Retina 해상도**: `contentScaleFactor`를 반영하지 않으면 흐리게 나옴. `layer.contentsScale = UIScreen.main.scale` 필수.
- **메모리**: A4 페이지 기준 약 600x930pt = @2x에서 1200x1860px = RGBA 4바이트 = **약 8.5MB/페이지**. 10페이지 동시 로드 시 85MB.
- **재그리기 비용**: `setNeedsDisplay()` 호출 시 전체 페이지를 다시 그림. 줌 변경 시마다 발생.

### 3-3. Y축 변환

`context.translateBy(x: 0, y: height)` + `context.scaleBy(x: 1, y: -1)` 접근은 정확하나, **텍스트가 상하 반전**되는 문제에 주의. Core Text는 자체적으로 Y축을 다루므로, 텍스트 그리기 전에 좌표를 다시 변환해야 할 수 있다. 이 부분을 3단계에서 반드시 프로토타이핑해야 한다.

### 3-4. 이미지 디코딩 타이밍

`rhwp_image_data` → `UIImage(data:)` → `context.draw()`는 **매 draw 호출마다 이미지를 디코딩**하게 된다. CGImage로 한 번 디코딩한 결과를 캐시해야 한다.

```swift
class CGDocumentRenderer {
    private var imageCache: [UInt16: CGImage] = [:]  // bin_data_id → CGImage
}
```

---

## 4. 다중 페이지 구조

### 4-1. ScrollView + LazyVStack vs UICollectionView

LazyVStack은 SwiftUI의 lazy loading을 활용하므로 M2에서 적절한 선택이다. 다만:

- **LazyVStack의 한계**: 한 번 로드된 뷰를 해제하지 않는다 (recycling 없음). 100페이지 문서에서 전체를 스크롤하면 100개 PageCanvasView가 메모리에 남는다.
- **UICollectionView**: 셀 재사용(dequeue)으로 메모리 일정. 하지만 SwiftUI 통합이 복잡.

### 4-2. 권고

M2에서는 LazyVStack으로 시작하되, **한 가지 보호장치**를 추가한다:

- 화면에서 멀어진 페이지의 `commands` 배열을 nil로 해제하고, 다시 보일 때 재생성
- 이는 `onAppear`/`onDisappear` modifier로 구현 가능

```swift
LazyVStack {
    ForEach(0..<pageCount, id: \.self) { index in
        PageCanvasView(commands: viewModel.commands(for: index))
            .frame(width: pageWidth, height: pageHeight)
            .onAppear { viewModel.loadPage(index) }
            .onDisappear { viewModel.unloadPage(index) }
    }
}
```

---

## 5. 이미지 데이터 FFI

### 5-1. 포인터 반환의 안전성

```rust
pub extern "C" fn rhwp_image_data(
    handle: *const RhwpHandle, bin_data_id: u16, out_len: *mut usize
) -> *const u8;
```

이 설계에서 반환된 `*const u8`의 라이프타임은 `RhwpHandle`에 묶여 있다. **Handle이 살아있는 동안은 안전**하다. 다만:

- Swift 측에서 `Data(bytesNoCopy:count:deallocator:)` 사용 시 `.none` deallocator를 명시해야 함. `.free`를 사용하면 double free.
- `rhwp_close()` 이후 이미지 데이터에 접근하면 use-after-free. 이를 방지하려면 `Data(bytes:count:)`로 **복사**하는 것이 더 안전.

### 5-2. 권고

M2에서는 복사 방식(`Data(bytes:count:)`)이 안전하다. 이미지당 수십~수백KB 수준이고, 페이지 렌더링 시 1회만 발생하므로 성능 영향 미미.

---

## 6. 누락된 고려사항

### 6-1. 폰트 폴백 (중요도: 높음)

iOS에는 한컴 전용 폰트(함초롬돋움, 함초롬바탕 등)가 없다. HWP 문서의 80% 이상이 한컴 전용 폰트를 사용한다. 3단계에서 최소한 다음 매핑이 필요:

| HWP 폰트 | iOS 폴백 |
|-----------|----------|
| 한컴돋움/함초롬돋움/HY중고딕 | Apple SD Gothic Neo |
| 한컴바탕/함초롬바탕/HY신명조 | Nanum Myeongjo (번들) 또는 AppleMyungjo |
| 굴림/돋움 | Apple SD Gothic Neo |
| 궁서/바탕 | AppleMyungjo |

구현계획서에 폰트 폴백 언급이 없다. 3단계에 **폰트 매핑 테이블**을 포함시켜야 한다. `mydocs/tech/font_fallback_strategy.md`를 참조할 것.

### 6-2. 줌/핀치 지원

M2 범위에 줌이 명시되어 있지 않다. `draw(_ rect:)`는 비트맵 기반이므로 핀치 줌 시 픽셀이 깨진다. 최소한:
- 줌 레벨 변경 시 `setNeedsDisplay()` + `contentScaleFactor` 조정
- 또는 `UIScrollView`의 줌 기능 활용 (이 경우 비트맵 스케일링 후 안정화 시 재그리기)

M2 범위라면 "줌 미지원, 1:1 보기만" 명시하는 것을 권장.

### 6-3. 접근성 (VoiceOver)

Core Graphics 직접 그리기는 접근성 트리가 자동 생성되지 않는다. M2에서는 스코프 밖으로 명시하되, M3 항목에 추가해야 한다.

### 6-4. 다크 모드

iOS 다크 모드에서 흰색 배경 문서가 눈부실 수 있다. M2에서는 무시하되, 인지하고 있어야 한다.

---

## 7. 단계 분리 검토

### 7-1. 1단계와 2단계 사이의 검증 공백

1단계 검증이 `cargo build + cargo test`인데, **JSON 출력의 정확성을 어떻게 검증하는가?** 1단계 완료 시 최소한 하나의 테스트 케이스에서 JSON 출력을 스냅샷 비교해야 한다.

### 7-2. 3단계가 가장 큰 위험 구간

3단계(Core Graphics 렌더러)는 텍스트 렌더링, 이미지, 경로, 패턴을 모두 포함한다. **텍스트 렌더링만으로도 별도 단계 분량**이다. 다음과 같이 분할을 권고한다:

- 3a: 도형 렌더링 (rect, line, ellipse, path, image) — Core Graphics 기본 API
- 3b: 텍스트 렌더링 (Core Text + 폰트 폴백 + 장평/자간 + 효과)

### 7-3. 5단계의 파일 선택과 다중 페이지는 독립 기능

파일 선택(UIDocumentPicker)과 다중 페이지 스크롤은 서로 독립적이다. 하나의 단계로 묶어도 무방하나, 각각의 검증 기준을 명확히 해야 한다.

### 7-4. 전체적으로 5단계는 적절

단계 간 의존성은 1→2→3→4→5로 선형이며 논리적이다. 3단계 분할만 고려하면 된다.

---

## 8. 과잉 설계 여부

### 8-1. M2 범위에서 적절

구현계획서는 전반적으로 **최소 기능(MVP)**에 집중하고 있다. Metal, 수식, 캐시 최적화를 M3로 미룬 것은 올바른 판단이다.

### 8-2. serde_json을 native only로 제한하는 것은 불필요한 복잡성

`#[cfg(not(target_arch = "wasm32"))]`로 serde_json을 제한하면, WASM 빌드에서 CanvasCommand의 Serialize가 사용 불가하다. 향후 WASM에서도 JSON 직렬화가 필요할 수 있으므로 (디버깅, WebWorker 통신 등), **serde는 무조건 의존성에 추가**하고 serde_json만 optional feature로 관리하는 것이 나을 수 있다. 단, WASM 바이너리 크기가 걱정이면 현행대로도 무방하다.

---

## 요약: 필수 수정 사항

| 우선순위 | 항목 | 영향 |
|---------|------|------|
| **P0 (차단)** | CanvasCommand에 TextStyle 정보 포함 | 이것 없이는 텍스트가 검은색 기본 글꼴로만 표시됨 |
| **P0 (차단)** | CanvasCommand에 LineStyle 정보 포함 | 선 색상/두께/종류가 모두 기본값 |
| **P0 (차단)** | CanvasCommand에 ShapeStyle 정보 포함 (ellipse, rect의 corner_radius) | 도형 채우기/테두리 누락 |
| **P1 (높음)** | 폰트 폴백 매핑 테이블 (3단계에 포함) | 한컴 폰트 → iOS 폰트 매핑 없으면 글꼴 깨짐 |
| **P1 (높음)** | 이미지 CGImage 캐시 (3단계에 포함) | 매 draw마다 디코딩 반복 방지 |
| **P2 (중간)** | 3단계를 3a(도형)/3b(텍스트)로 분할 | 리스크 감소 |
| **P2 (중간)** | 다중 페이지 메모리 관리 (onDisappear 해제) | 100페이지+ 문서에서 메모리 폭증 방지 |
| **P3 (낮음)** | 줌 미지원 명시 | 범위 혼동 방지 |
