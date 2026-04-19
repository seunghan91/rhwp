# Stage 3 완료 보고서 — Task M100 #186
# 표 직렬화 (Control::Table → hp:tbl + Control dispatcher)

## 완료 일자
2026-04-18

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/table.rs` | 신규: write_table, write_cell, write_cell_para |
| `src/serializer/hwpx/section.rs` | render_paragraph_runs pub(crate) 공개, \u{0002} Control dispatcher 추가, run 빌딩 구조 재설계 |
| `src/serializer/hwpx/mod.rs` | `pub mod table;` 추가, Stage 3 테스트 4개 추가 |

## 구현 요약

### table.rs (신규)

```rust
pub fn write_table(out: &mut String, tbl: &Table, default_tab_width: u32)
```

출력 구조:
```xml
<hp:tbl id=".." rowCnt=".." colCnt=".." borderFillIDRef=".." ...>
  <hp:sz width=".." widthRelTo="ABSOLUTE" .../>
  <hp:pos treatAsChar=".." vertRelTo=".." horzRelTo=".." .../>
  <hp:outMargin left=".." right=".." top=".." bottom=".."/>
  <hp:inMargin left=".." right=".." top=".." bottom=".."/>
  [<hp:cellzone .../> ...]
  <hp:tr>
    <hp:tc header=".." borderFillIDRef="..">
      <hp:subList vertAlign="..">
        <hp:p ...><hp:run .../></hp:p>
      </hp:subList>
      <hp:cellAddr colAddr=".." rowAddr=".."/>
      <hp:cellSpan colSpan=".." rowSpan=".."/>
      <hp:cellSz width=".." height=".."/>
      <hp:cellMargin left=".." right=".." top=".." bottom=".."/>
    </hp:tc>
  </hp:tr>
</hp:tbl>
```

- 빈 셀: `<hp:run charPrIDRef="0"/>` (self-closing)
- 텍스트 셀: `render_paragraph_runs` 재사용 + `<hp:linesegarray>`

### render_paragraph_runs 재설계 (section.rs)

기존: 단순 `<hp:t>` 문자열 누적 → 마지막에 run 닫기

신규: 3-버퍼 접근법
- `ctrl_section`: `\u{0002}` 만나면 Control XML 누적 (→ `<hp:t>` 이전에 위치)
- `t_content`: 탭/줄바꿈/텍스트 누적 (→ `<hp:t>` 내용)
- `char_buf`: xml_escape 대기

`emit_run!()` 매크로로 현재 run 출력:
```
<hp:run charPrIDRef="N">[ctrl_section]<hp:t>[t_content]</hp:t></hp:run>
(또는 t_content가 없으면 <hp:t/>)
```

`\u{0002}` 처리:
1. 이미 텍스트가 있으면 현재 run emit 후 새 run 시작
2. 대응 Control::Table → write_table → ctrl_section에 누적
3. utf16_pos += 1

### pub(crate) 공개
`render_paragraph_runs`를 `pub(crate)`로 공개하여 `table.rs`의 `write_cell_para`에서 재사용.

## 테스트 결과

| 테스트 | 결과 |
|--------|------|
| `empty_table_roundtrip` | ✅ 2×3 표 row/col count 보존 |
| `table_cell_text_roundtrip` | ✅ 셀 텍스트 직렬화 + 라운드트립 |
| `table_borderfillidref` | ✅ borderFillIDRef=7 출력 확인 |
| `table_cellspan_roundtrip` | ✅ colSpan=2 보존 |
| 기존 36개 | ✅ 회귀 없음 |
| **합계** | **40 passed** |

## 주요 설계 결정

- `<hp:tbl>`은 `<hp:run>` 내에서 `<hp:t>` 이전에 위치 (ref_table.hwpx 분석 기반)
- 첫 문단의 controls에 secPr/colPr control도 포함됨 → 테스트는 type으로 Table 탐색
- 셀 문단 렌더링은 `render_paragraph_runs` 재사용으로 중첩 표 지원 가능
