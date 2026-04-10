# Task #93 — 4단계 완료보고서

## DocumentView 교체 (WKWebView → Core Graphics) ✅

### 작업 내용

1. **DocumentViewModel** — `svgContent: String` → `pageTree: RenderNode?` 전환
2. **DocumentView** — `SVGWebView` → `PageCanvasView` 교체
3. **Y축 좌표계 문제 해결** — UIKit CGContext 좌표계 이해 + 국소 재반전 패턴

### Y축 좌표계 해결 과정

| 시도 | 결과 | 원인 |
|------|------|------|
| 전체 `translateBy + scaleBy(y:-1)` | 텍스트/이미지 180도 회전 | CTM 음수 scaleY가 Core Text/CGImage에 전파 |
| 전체 변환 제거 + 개별 flipY | 작동하지만 코드 복잡 | 좌표계 불일치 |
| **UIKit CGContext 그대로 사용** | ✅ **정상** | UIView.draw()는 이미 좌상단 원점 |

**최종 해결**: `UIView.draw(_ rect:)`의 CGContext는 UIKit이 이미 `translateBy(y: height) + scaleBy(y: -1)`을 적용한 상태. 렌더 트리 좌표(좌상단 원점)를 그대로 사용. **Core Text와 CGImage만 bbox 영역 내에서 국소 Y반전**.

### 검증 결과 (iPad Simulator)

- 텍스트 (한글/영문): ✅ 올바른 방향, 폰트 폴백 적용
- 이미지 (nipa 로고): ✅ 정상 표시
- 도형 (수평선): ✅ 올바른 위치/색상
- 레이아웃: ✅ SVG 출력과 동일한 배치
- 페이지 정보: ✅ 1/66쪽 (793×1122pt)
- 크래시/에러: ✅ 없음
