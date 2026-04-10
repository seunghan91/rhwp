# Task #93 — 3b단계 완료보고서

## Core Text 텍스트 렌더링 + 폰트 폴백 매핑 ✅

### 작업 내용

1. **CGTreeRenderer 텍스트 렌더링** — Core Text 기반으로 교체
2. **`FontFallback.swift`** — HWP 폰트 → iOS 시스템 폰트 매핑 (30+ 항목)
3. **각주 마커 렌더링** — 위첨자 55% 크기

### Core Text 텍스트 처리

| 속성 | 구현 |
|------|------|
| 폰트 | `CTFontCreateWithName` + 폴백 매핑 |
| Bold/Italic | `CTFontCreateCopyWithSymbolicTraits` |
| 장평 (ratio) | `CGAffineTransform(scaleX:)` → `CTFontCreateCopyWithAttributes` |
| 자간 (letter_spacing) | `.kern` NSAttributedString attribute |
| 색상 | `.foregroundColor` → `colorRefToCGColor` |
| 밑줄 | 베이스라인 아래 직선 (색상/형태별) |
| 취소선 | 중앙 직선 (색상/형태별) |
| 음영 (형광펜) | 배경 반투명 사각형 |
| Y축 텍스트 반전 | `translateBy` + `scaleBy(y: -1)` |

### 폰트 폴백 매핑

| HWP 폰트 분류 | iOS 폴백 |
|---------------|----------|
| Serif (한컴바탕, 함초롬바탕, 바탕, HY신명조 등) | AppleMyungjo |
| Sans (한컴돋움, 함초롬돋움, 굴림, 맑은고딕 등) | AppleSDGothicNeo |
| Sans Bold (HY견고딕, HY헤드라인M) | AppleSDGothicNeo-Bold |
| 영문 Serif (Times New Roman) | TimesNewRomanPSMT |
| 영문 Sans (Arial, Calibri, Tahoma) | ArialMT / Helvetica Neue |
| 영문 Mono (Courier New) | CourierNewPSMT |

### 미구현 (M3 이후)

| 항목 | 비고 |
|------|------|
| 외곽선 (outline_type) | stroke + fill 조합 |
| 그림자 (shadow_type) | `context.setShadow` |
| 양각/음각 (emboss/engrave) | 색상 오프셋 |
| 강조점 (emphasis_dot) | 글자 위 점 그리기 |
| 탭 리더 (tab_leaders) | 채움 기호 |
| 이중선/삼중선 밑줄 | line_shape별 분기 |

### 검증 결과

- Xcode 빌드 (iPad Simulator): ✅ BUILD SUCCEEDED
