# Stage 2 완료 보고서 — Task M100 #186
# 다중 run 분할 (char_shapes UTF-16 offset 기반)

## 완료 일자
2026-04-18

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/section.rs` | `TEXT_SLOT` → `TEXT_RUN_SLOT`, `render_paragraph_parts` → `render_paragraph_runs` |
| `src/serializer/hwpx/mod.rs` | Stage 2 테스트 2개 추가 |

## 구현 요약

### 상수 변경

```rust
// Before (Stage 1)
const TEXT_SLOT: &str = "<hp:t/>";

// After (Stage 2)
const TEXT_RUN_SLOT: &str = r#"<hp:run charPrIDRef="0"><hp:t/></hp:run>"#;
```

`TEXT_RUN_SLOT`은 템플릿의 두 번째 `<hp:run>` 전체(secPr run 제외)에 매칭된다. 이를 교체함으로써 charPrIDRef도 동시에 처리한다.

### render_paragraph_runs

`render_paragraph_parts`(단일 `<hp:t>` 생성)를 대체하는 새 함수:

```rust
fn render_paragraph_runs(
    para: &Paragraph,
    vert_start: u32,
    default_tab_width: u32,
) -> (String, String, u32)  // (runs_xml, linesegs_xml, next_vert)
```

- `para.char_shapes`를 `start_pos` 기준으로 run 경계 판단
- UTF-16 위치가 `shapes[i+1].start_pos`에 도달하면 현재 run을 닫고 새 run을 열기
- 탭/줄바꿈/제어문자 처리는 기존과 동일
- char_shapes가 비어있으면 `charPrIDRef="0"` 단일 run

### write_section 변경

```rust
// Before: TEXT_SLOT 교체 + charPrIDRef 별도 치환 (broken)
let mut out = EMPTY_SECTION_XML.replacen(TEXT_SLOT, &first_t, 1);
out.replacen(r#"charPrIDRef="0"><hp:t/>"#, ...);  // TEXT_SLOT 교체 후 이미 없는 패턴

// After: TEXT_RUN_SLOT 전체 교체
let (first_runs, first_linesegs, _) = render_paragraph_runs(p, ...);
let out = EMPTY_SECTION_XML.replacen(TEXT_RUN_SLOT, &first_runs, 1);
```

broken charPrIDRef 치환 코드 제거됨.

## 테스트 결과

| 테스트 | 결과 |
|--------|------|
| `single_charshape_single_run` | ✅ charPrIDRef="7" 정상 출력 |
| `multi_run_splits_correctly` | ✅ run 3개 올바르게 분리 (charPrIDRef 1/2/3) |
| 기존 serializer::hwpx 34개 | ✅ 회귀 없음 |
| **합계** | **36 passed** |

## 부수 수정

- `charPrIDRef` 치환이 텍스트가 있는 첫 문단에서 silent fail하던 버그 수정
- 추가 문단의 단일 `<hp:run>` 래퍼도 `render_paragraph_runs`로 통합 (charPrIDRef 정확도 향상)
