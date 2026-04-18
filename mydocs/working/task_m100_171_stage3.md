---
# Stage 3 완료 보고서 — Task M100 #171
# paraPr + tabPr + borderFills 직렬화
---

## 완료 일시
2026-04-18

## 작업 내용

### 변경 파일
| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/utils.rs` | `border_line_type_to_str`, `border_width_to_str`, `alignment_to_str`, `line_spacing_type_to_str` 추가 |
| `src/serializer/hwpx/header.rs` | 3개 placeholder 함수 → 동적 생성으로 교체 |

### 구현 내용

#### utils.rs 헬퍼 4개 추가
- `border_line_type_to_str(BorderLineType) -> &str` — 18종 테두리선 역매핑
- `border_width_to_str(u8) -> &str` — 굵기 인덱스 → "N.N mm"
- `alignment_to_str(Alignment) -> &str` — 6종 정렬 역매핑
- `line_spacing_type_to_str(LineSpacingType) -> &str` — 4종 줄간격 역매핑

#### header.rs — borderFills 동적 생성
- `write_border_fills(border_fills)`: 비어 있으면 기존 2개 placeholder, 있으면 IR 전체 직렬화
  - ID = index + 1 (1-based)
  - 4방향 테두리선 (type/width/color)
  - diagonal 타입/색상
  - FillType::Solid → `<hc:winBrush faceColor/hatchColor/alpha>`

#### header.rs — tabPr 동적 생성
- `write_tab_properties(tab_defs)`: 비어 있으면 placeholder (ID=0)
  - autoTabLeft/autoTabRight 직렬화
  - `<hh:tabItem pos/type/leader/>` 직접 출력 (switch 없이 raw HWPUNIT)

#### header.rs — paraPr 동적 생성
- `write_para_properties(para_shapes)`: 비어 있으면 기본값 1개
- `write_single_para_pr(id, ps)`:
  - `tabPrIDRef`, `align`, `heading(type/idRef/level)`
  - `breakSetting`: attr2 비트 추출 (widowOrphan/keepWithNext/keepLines/pageBreakBefore)
  - `autoSpacing`: attr1 비트 추출 (eAsianEng/eAsianNum)
  - `<hh:margin left/right/indent/prev/next>` 직접 attribute 방식 (단위 변환 없음)
  - `<hh:lineSpacing type/value>` 직접 값 출력
  - `<hh:border borderFillIDRef/offsetLeft/Right/Top/Bottom>`

### 단위 테스트 7개 신규 추가 (모두 통과)
| 테스트 | 검증 내용 |
|--------|----------|
| `empty_border_fills_emits_placeholder` | 빈 border_fills → placeholder 2개 |
| `border_fill_roundtrip` | borderFill IR → serialize → parse → 선 종류/색상/fill 보존 |
| `empty_tab_defs_emits_placeholder` | 빈 tab_defs → placeholder |
| `tab_def_auto_tab_roundtrip` | autoTabLeft/Right 직렬화 라운드트립 |
| `tab_items_roundtrip` | tabItem pos/type/leader 직렬화 라운드트립 |
| `empty_para_shapes_emits_default_parappr` | 빈 para_shapes → placeholder |
| `para_shape_roundtrip` | margin/lineSpacing/alignment 라운드트립 |

## 테스트 결과

```
running 27 tests (serializer::hwpx 전체)
... 27 passed; 0 failed
```

## 특이사항

- paraPr의 margin/lineSpacing은 impl plan의 switch/HwpUnitChar 방식 대신 **직접 attribute 방식** 사용
  - 파서가 `<hh:margin left="..."/>` 형식도 정상 파싱하므로 라운드트립 동일하게 보장
  - HwpUnitChar 2× 스케일 변환 불필요
  - tabItem position도 동일하게 raw HWPUNIT 직접 출력

---

> **Stage 3 완료 승인 요청**: 위 내용 확인 후 승인해 주시면 **Stage 4 (styles + numberings + section.xml ID 연동)** 를 시작하겠습니다.
