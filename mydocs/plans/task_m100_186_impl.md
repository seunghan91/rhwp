# 구현계획서 — Task M100 #186
# HWPX section.xml 완전 동적화 (secPr + 다중 run + Control dispatcher)

## 참조 구조 (ref_table.hwpx 역공학)

```xml
<!-- 표가 있는 문단 구조 -->
<hp:p paraPrIDRef="N" styleIDRef="N">
  <hp:run charPrIDRef="N">           <!-- ← tbl은 run 안에 위치 -->
    <hp:tbl rowCnt="N" colCnt="N" borderFillIDRef="N" ...>
      <hp:sz width="N" height="N" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE" protect="0"/>
      <hp:pos treatAsChar="0/1" vertRelTo="PARA" horzRelTo="COLUMN" .../>
      <hp:outMargin left right top bottom/>
      <hp:inMargin left right top bottom/>
      <hp:cellzone startColAddr endColAddr startRowAddr endRowAddr borderFillIDRef/>
      <hp:tr>
        <hp:tc borderFillIDRef="N" ...>
          <hp:subList textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP/CENTER/BOTTOM" ...>
            <hp:p ...><hp:run charPrIDRef="N"/><hp:linesegarray>...</hp:linesegarray></hp:p>
          </hp:subList>
          <hp:cellAddr colAddr rowAddr/>
          <hp:cellSpan colSpan rowSpan/>
          <hp:cellSz width height/>
          <hp:cellMargin left right top bottom/>
        </hp:tc>
      </hp:tr>
    </hp:tbl>
    <hp:t/>   <!-- 선택적 trailing 텍스트 -->
  </hp:run>
  <hp:linesegarray>...</hp:linesegarray>
</hp:p>
```

## Stage 1 — secPr/pagePr 동적화

**파일**: `src/serializer/hwpx/section.rs`

### 변경 내용

현재 `empty_section0.xml` 템플릿의 secPr run에서 pagePr 부분만 IR 값으로 치환한다.

**치환 대상 (고유 패턴)**:

| 패턴 | 대체값 출처 |
|------|-----------|
| `landscape="WIDELY"` | `page_def.landscape` → WIDELY / NARROW |
| `width="59528"` | `page_def.width` |
| `height="84186"` | `page_def.height` |
| `gutterType="LEFT_ONLY"` | `binding` → LEFT_ONLY / RIGHT_ONLY / TOP_ONLY / NONE |
| `header="4252" footer="4252"` | `margin_header`, `margin_footer` |
| `gutter="0"` | `margin_gutter` |
| `left="8504" right="8504"` | `margin_left`, `margin_right` |
| `top="5668" bottom="4252"` | `margin_top`, `margin_bottom` |

**구현**: `substitute_page_def(out: &mut String, page_def: &PageDef)` 함수  
- 대상 패턴이 모두 고유하므로 `replace()` 사용  
- `default_tab_spacing` 치환도 포함 (`tabStop="8000"` → 실제 값)

### 테스트
- `pagePr_dynamic_width_height`: 직렬화 → parse → width/height 보존
- `pagePr_margins_dynamic`: left/right/top/bottom/header/footer 보존
- `pagePr_landscape_narrow`: landscape=false → `NARROW`

---

## Stage 2 — 다중 run 분할

**파일**: `src/serializer/hwpx/section.rs`

### 변경 내용

현재 `render_paragraph_parts`는 단일 run을 생성한다. `char_shapes`를 기준으로 구간을 나눠 여러 `<hp:run>`을 생성한다.

**알고리즘**:

```
char_shapes를 start_pos(UTF-16 offset) 기준으로 정렬
구간: [shape[0].start_pos .. shape[1].start_pos), [shape[1].start_pos .. shape[2].start_pos), ...
text를 UTF-16 offset 기준으로 순회하면서
  현재 위치가 다음 구간 시작점을 넘으면:
    현재 run 닫기 (</hp:run>)
    새 run 열기 (<hp:run charPrIDRef="N">)
탭/줄바꿈/제어문자 처리는 기존 그대로
```

**함수 시그니처 변경**:
```rust
fn render_paragraph_runs(
    para: &Paragraph,
    vert_start: u32,
    default_tab_width: u32,
) -> (String, String, u32)
```
반환: (runs_xml, linesegs_xml, next_vert)

### 테스트
- `multi_run_splits_correctly`: char_shapes 2개 → `<hp:run>` 2개 출력
- `single_charshape_single_run`: char_shapes 1개 → run 1개

---

## Stage 3 — 표 직렬화

**신규 파일**: `src/serializer/hwpx/table.rs`  
**수정 파일**: `src/serializer/hwpx/section.rs`

### table.rs

```rust
pub fn write_table(out: &mut String, table: &Table) {
    // <hp:tbl ...> 속성들
    // <hp:sz> <hp:pos> <hp:outMargin> <hp:inMargin>
    // <hp:cellzone> for zone in table.zones
    // 행 그룹별 <hp:tr>
    //   for cell in row_cells { write_cell(out, cell) }
    // </hp:tbl>
}

fn write_cell(out: &mut String, cell: &Cell) {
    // <hp:tc ...>
    //   <hp:subList vertAlign=...>
    //     for para in cell.paragraphs { write_cell_para(out, para) }
    //   </hp:subList>
    //   <hp:cellAddr/> <hp:cellSpan/> <hp:cellSz/> <hp:cellMargin/>
    // </hp:tc>
}

fn write_cell_para(out: &mut String, para: &Paragraph)
// 셀 내 문단: 재귀 가능, 표 중첩 포함
```

### 행 분리 로직

`table.cells`는 행 우선 순서. `row_count × col_count` 그리드에서 병합셀은 `cell_grid`가 None인 인덱스 건너뜀.

```
현재 row = 0
for cell in table.cells:
    if cell.row != current_row:
        </hp:tr><hp:tr>
        current_row = cell.row
    write_cell(out, cell)
```

### Control dispatcher (section.rs)

`render_paragraph_runs` 에서 `\u{0002}` 문자를 만나면:
```
ctrl_idx 증가
현재 run이 열려있으면 텍스트/t 출력 후 닫기
match para.controls[ctrl_idx]:
    Control::Table(tbl) => write_table(out, tbl)
    _ => (무시)
다음 run 다시 열기 (남은 텍스트가 있으면)
```

### 테스트
- `empty_table_roundtrip`: 2×2 표 → serialize → parse → row/col count 보존
- `table_cell_text_roundtrip`: 셀 내 텍스트 → serialize → parse → 텍스트 보존
- `table_borderfillidref`: borderFillIDRef 직렬화
- `table_cellspan_roundtrip`: colSpan/rowSpan 보존
- `form002_table_output`: form-002.hwpx 라운드트립 → `<hp:tbl>` 존재 확인

---

## Stage 4 — 그림 직렬화 + BinData

**신규 파일**: `src/serializer/hwpx/picture.rs`  
**수정 파일**: `src/serializer/hwpx/mod.rs`, `src/serializer/hwpx/content.rs`

### BinData 수집

`serialize_hwpx`에서 Document를 순회하며 `Control::Picture` 발견 시:
- `image_attr.bin_data_id` → `doc.doc_info.bin_data[id]`에서 실제 바이트 획득
- ZIP에 `BinData/image{id}.{ext}` 저장
- `BinDataEntry { id: "image{id}", href: "BinData/image{id}.{ext}", media_type }` 수집
- `write_content_hpf(section_hrefs, bin_data_entries)` 호출 시 manifest에 포함

### picture.rs

```rust
pub fn write_picture(out: &mut String, pic: &Picture) {
    // <hp:pic zOrder textWrap instid ...>
    //   <hp:sz width height .../>
    //   <hp:pos treatAsChar vertRelTo horzRelTo .../>
    //   <hp:outMargin .../>
    //   <hp:inMargin .../>
    //   <hp:imgClip left right top bottom/>
    //   <hc:img binaryItemIDRef="image{bin_data_id}" bright contrast effect/>
    // </hp:pic>
}
```

### Control dispatcher 확장

table dispatcher와 동일한 구조로 `Control::Picture` 분기 추가.

### 테스트
- `picture_bindata_entry_generated`: Picture 있는 문서 → `BinData/` 항목 생성
- `picture_manifest_entry`: content.hpf에 opf:item 출력
- `picture_roundtrip_attr`: binaryItemIDRef/크기/위치 보존

---

## Stage 5 — 통합 검증

### 검증 케이스

| 파일 | 확인 내용 |
|------|----------|
| `rt_ref_table.hwpx` | 2×3 표 내용 한컴 표시 |
| `rt_form002.hwpx` | 26×27 표 본문 표시 (form-002.hwpx 라운드트립) |
| `rt_ref_text.hwpx` | 회귀 없음 |
| `rt_ref_mixed.hwpx` | 회귀 없음 |

### IrDiff 기준
- `rhwp ir-diff rt_ref_table.hwpx samples/hwpx/ref/ref_table.hwpx` → diff 0 목표 (표 구조)

---

## 구현 의존 관계

```
Stage 1 (pagePr)
    ↓
Stage 2 (multi-run)
    ↓
Stage 3 (table)  ← table.rs 신규
    ↓
Stage 4 (picture) ← picture.rs 신규
    ↓
Stage 5 (검증)
```

각 Stage는 이전 Stage 완료 후 착수.
