---
# 최종 결과 보고서 — Task M100 #171
# header.xml IR 기반 직렬화 (글꼴/스타일/문단 모양)
---

## 이슈
[#171](https://github.com/edwardkim/rhwp/issues/171) — header.xml IR 기반 직렬화

## 브랜치
`feature/task171-header-xml-ir` (7 commits)

## 기간
2026-04-18

## 마일스톤
M100 (v1.0.0)

---

## 목표

`Document` IR의 DocInfo(글꼴/글자모양/문단모양/스타일/번호매기기/테두리 등)를  
`Contents/header.xml`에 동적으로 직렬화한다.  
기존 정적 template(`EMPTY_HEADER_XML`) + 플레이스홀더 방식을 완전히 교체.

---

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/header.rs` | 4단계 직렬화 함수 구현 (Stage 1~4) |
| `src/serializer/hwpx/section.rs` | ID 동적 연동 + tab_extended 원본값 복원 |
| `src/serializer/hwpx/utils.rs` | 역매핑 헬퍼 6개 추가 |

---

## 단계별 구현 내용

### Stage 1 — fontfaces 직렬화
- `write_fontfaces`: 7개 언어 그룹 동적 생성, 글꼴 이름 XML escape

### Stage 2 — charPr 직렬화
- `write_char_properties` / `write_single_char_pr`
- bold/italic/underline/strikeout/outline/shadow/emboss/engrave/sup/sub 조건부 출력
- `color_ref_to_hex`, `line_shape_to_str` 헬퍼

### Stage 3 — borderFills / tabPr / paraPr 직렬화
- `write_border_fills`: 4방향 테두리 + diagonal + FillType::Solid
- `write_tab_properties`: autoTabLeft/Right + tabItem pos/type/leader
- `write_para_properties` / `write_single_para_pr`: margin/lineSpacing/alignment/breakSetting/autoSpacing/border
- `border_line_type_to_str`, `border_width_to_str`, `alignment_to_str`, `line_spacing_type_to_str` 헬퍼

### Stage 4 — styles / numberings + section.xml ID 연동
- `write_styles`: PARA/CHAR 타입, paraPrIDRef/charPrIDRef/nextStyleIDRef
- `write_numberings`: 7레벨 paraHead(numFormat/charPrIDRef/text/start)
- section.rs: `paraPrIDRef`/`styleIDRef`/`charPrIDRef` Paragraph IR 기반 동적 주입

### 추가 수정 — 탭 폭 원본값 복원
- `tab_extended[i][0]` 우선 사용 → 없으면 `default_tab_spacing` 폴백
- 라운드트립 시 원본 탭 폭(예: 3028) 그대로 복원

---

## 단위 테스트

| 단계 | 신규 테스트 수 | 누적 |
|------|-------------|------|
| Stage 1 | 4 | 14 |
| Stage 2 | 6 | 20 |
| Stage 3 | 7 | 27 |
| Stage 4 | 4 | 31 |

**최종: 31 passed; 0 failed** (serializer::hwpx 전체)

---

## 라운드트립 검증

| 파일 | 원본 | 결과 |
|------|------|------|
| `rt_ref_text.hwpx` | ref_text.hwpx (1문단) | 텍스트 정상 |
| `rt_ref_mixed.hwpx` | ref_mixed.hwpx (4문단, 탭/줄바꿈) | 탭 폭 원본 복원(3028) |
| `rt_form002.hwpx` | form-002.hwpx (표 포함) | 텍스트 8문단, 표 내부는 #172 |

---

## 특이사항

- paraPr margin/lineSpacing: impl plan의 HwpUnitChar switch 방식 대신 직접 attribute 방식 사용 (파서 호환)
- numFormat: 파서가 `parse_u8("DIGIT")` → 0으로 저장하므로 0→"DIGIT" 역매핑으로 라운드트립 보장
- form-002.hwpx 내용 미표시: 전체 본문이 26×27 표 안에 있음 → #172(표 직렬화) 이후 해결
