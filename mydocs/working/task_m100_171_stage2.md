---
# Stage 2 완료 보고서 — Task M100 #171
# charPr 직렬화
---

## 완료 일시
2026-04-18

## 작업 내용

### 변경 파일
| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/utils.rs` | `color_ref_to_hex()`, `line_shape_to_str()` 추가 |
| `src/serializer/hwpx/header.rs` | `char_properties_placeholder` 제거 → `write_char_properties()` 동적 생성으로 교체 |

### 구현 내용

#### utils.rs 헬퍼
- `color_ref_to_hex(c: u32) -> String` — `0x00BBGGRR` → `#RRGGBB` / `0xFFFFFFFF` → `"none"`
- `line_shape_to_str(shape: u8) -> &'static str` — 선 종류 13종 역매핑 (SOLID/DASH/DOT/...)

#### header.rs — charPr 직렬화
- `write_char_properties(out, char_shapes)` — `char_shapes` 비어 있으면 기본값 1개, 있으면 전체 직렬화
- `write_single_char_pr(out, id, cs)` — 단일 `<hh:charPr>` 생성
  - 필수 속성: `height`, `textColor`, `shadeColor`, `useKerning`, `borderFillIDRef`
  - 자식 요소: `fontRef`, `ratio`, `spacing`, `relSz`, `offset` (7-tuple)
  - 조건부: `<hh:bold/>`, `<hh:italic/>` (true인 경우만)
  - 조건부: `<hh:underline>` (underline_type != None인 경우, type/shape/color 포함)
  - 조건부: `<hh:strikeout>` (strikethrough == true인 경우)
  - 조건부: `<hh:outline>`, `<hh:shadow>`, `<hh:emboss>`, `<hh:engrave>`, `<hh:supscript>`, `<hh:subscript>`
- `lang7_attr`, `lang7_attr_u8`, `lang7_attr_i8` — 7-tuple 속성 출력 헬퍼

### 단위 테스트 6개 신규 추가 (모두 통과)
| 테스트 | 검증 내용 |
|--------|----------|
| `empty_charshapes_emits_default_charpr` | 빈 char_shapes → ID 0 기본값 출력 |
| `charpr_bold_italic_emitted` | bold/italic=true → `<hh:bold/>`, `<hh:italic/>` 존재 |
| `charpr_no_bold_italic_when_false` | bold/italic=false → 요소 미출력 |
| `charpr_underline_bottom_emitted` | underline_type=Bottom → `type="BOTTOM"` 확인 |
| `charpr_strikeout_emitted` | strikethrough=true → `<hh:strikeout>` 확인 |
| `charpr_roundtrip` | serialize → parse_hwpx_header → IR 동등성 확인 |

## 테스트 결과

```
running 20 tests (serializer::hwpx 전체)
... 20 passed; 0 failed
```

기존 20개 포함 전체 통과, 회귀 없음.

---

> **Stage 2 완료 승인 요청**: 위 내용 확인 후 승인해 주시면 **Stage 3 (paraPr + tabPr + borderFills 직렬화)** 를 시작하겠습니다.
