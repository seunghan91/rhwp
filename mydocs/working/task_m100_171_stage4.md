---
# Stage 4 완료 보고서 — Task M100 #171
# styles + numberings 직렬화 + section.xml ID 동적 연동
---

## 완료 일시
2026-04-18

## 작업 내용

### 변경 파일
| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/header.rs` | `styles_placeholder` → `write_styles`, numberings 빈 태그 → `write_numberings` 교체 |
| `src/serializer/hwpx/section.rs` | 첫 문단 + 추가 문단 ID 하드코딩 → Paragraph IR 기반 동적 연동 |

### 구현 내용

#### header.rs — write_styles 동적 생성
- `write_styles(out, styles: &[Style])`: 비어 있으면 `바탕글` 1개 placeholder
- 있으면 IR 전체 직렬화: `id`, `type(PARA/CHAR)`, `name`, `engName`, `paraPrIDRef`, `charPrIDRef`, `nextStyleIDRef`
- 글자모양 스타일: `style_type == 1` → `type="CHAR"`

#### header.rs — write_numberings 동적 생성
- `write_numberings(out, numberings: &[Numbering])`: 비어 있으면 `itemCnt="0"` 빈 태그
- 있으면 7레벨 `<hh:paraHead>` 직렬화: `level`, `start`, `text`, `numFormat`, `charPrIDRef`
- `num_format_to_str(u8)`: 0→DIGIT, 1→CIRCLED_DIGIT, 2→HANGUL_LETTER, ...

#### section.rs — 동적 ID 연동
- 첫 문단: 템플릿의 `paraPrIDRef="0"` → `para.para_shape_id`, `styleIDRef="0"` → `para.style_id`
- 텍스트 run(`charPrIDRef="0"><hp:t/>` 고유 패턴) → `para.char_shapes[0].char_shape_id`
- 추가 문단: `<hp:p>` 태그와 `<hp:run>` 태그 format 매크로에서 직접 실제 ID 주입

### 단위 테스트 4개 신규 추가 (모두 통과)
| 테스트 | 검증 내용 |
|--------|----------|
| `empty_styles_emits_placeholder` | 빈 styles → 바탕글 placeholder |
| `style_roundtrip` | Style IR → serialize → parse → 이름/IDRef 보존 |
| `empty_numberings_emits_empty_tag` | 빈 numberings → `itemCnt="0"` 빈 태그 |
| `numbering_roundtrip` | Numbering IR → serialize → parse → start/level 보존 |

## 테스트 결과

```
running 31 tests (serializer::hwpx 전체)
... 31 passed; 0 failed
```

## 라운드트립 검증 파일 재생성
- `output/verify/rt_ref_text.hwpx` — ref_text.hwpx 라운드트립 (sections=1, para=1)
- `output/verify/rt_ref_mixed.hwpx` — ref_mixed.hwpx 라운드트립 (sections=1, para=4)
- `output/verify/rt_form002.hwpx` — form-002.hwpx 라운드트립 (sections=1, para=8)

## 특이사항

- section.xml 첫 문단 텍스트 run의 `charPrIDRef`: 고유 패턴 `charPrIDRef="0"><hp:t/>` 로 정확히 두 번째 run만 치환
- secPr run의 charPrIDRef는 변경하지 않음 (구조적 run이므로 ID 무관)
- numFormat 파서가 `parse_u8("DIGIT")` → 0으로 저장하므로 0→"DIGIT" 역매핑이 라운드트립 보장

---

> **Stage 4 완료 승인 요청**: 위 내용 확인 후 `output/verify/` 파일들을 한컴에서 열어 탭 폭/스타일 표현을 확인해 주시면 **최종 결과 보고서 작성 및 PR 준비**를 진행하겠습니다.
