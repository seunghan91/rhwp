---
# 구현계획서 — Task M100 #171
# header.xml IR 기반 직렬화 (글꼴/스타일/문단 모양)

## 전략

파서(`src/parser/hwpx/header.rs`)가 XML → IR로 변환하는 매핑을 정확히 역전(IR → XML)하여 `write_header()`를 동적 생성 방식으로 교체한다.  
`String` 빌더 방식으로 XML을 직접 생성한다 (quick-xml writer 불필요 — 파서와 일관성 유지).

## 단계 구성 (4단계)

| 단계 | 내용 | 목표 |
|------|------|------|
| 1 | XML 기반 구조 + fontfaces 직렬화 | `write_header()` 교체, 글꼴 라운드트립 |
| 2 | charPr 직렬화 | 글자모양 라운드트립 |
| 3 | paraPr + tabPr + borderFills 직렬화 | 문단모양 라운드트립 |
| 4 | styles + numberings + section.xml ID 연동 | 전체 ID 라운드트립 |

---

## Stage 1: XML 기반 구조 + fontfaces 직렬화

**파일**: `src/serializer/hwpx/header.rs`

### 구현 내용

1. `write_header(doc: &Document)` — `EMPTY_HEADER_XML` include_str 제거, 동적 생성으로 교체
2. XML 선언 + `<hh:head>` 열기 (네임스페이스 7개 고정)
3. `<hh:beginNum page="1" .../>` (하드코딩 기본값)
4. `<hh:refList>` → `<hh:fontfaces itemCnt="7">` 생성

```
<hh:fontfaces itemCnt="7">
  <hh:fontface lang="HANGUL" fontCnt="{n}">
    <hh:font id="{i}" face="{name}" type="TTF" isEmbedded="0">
      <hh:typeInfo .../>
    </hh:font>
    ...
  </hh:fontface>
  ...  (LATIN/HANJA/JAPANESE/OTHER/SYMBOL/USER)
</hh:fontfaces>
```

lang 순서: `["HANGUL","LATIN","HANJA","JAPANESE","OTHER","SYMBOL","USER"]` (파서와 동일 인덱스).  
`<hh:typeInfo>`는 기본값으로 고정 (`familyType="FCAT_GOTHIC" weight="6" ...`).

5. `<hh:refList>` 닫기는 뒤 단계에서 추가 → Stage 1에서는 fontfaces 이후 나머지는 빈 섹션으로 placeholder

### 단위 테스트

- 빈 `Document`(기본 DocInfo) → serialize → ZIP 추출 → `<hh:fontfaces itemCnt="7">` 존재 확인
- 글꼴 2개 있는 DocInfo → `<hh:font face="HY고딕">` 포함 확인
- 파서 라운드트립: `parse_hwpx_header(serialize) == original doc_info.font_faces`

---

## Stage 2: charPr 직렬화

**파일**: `src/serializer/hwpx/header.rs`

### 구현 내용

`<hh:charProperties>` 블록 생성:

```xml
<hh:charProperties>
  <hh:charPr id="{i}" height="{base_size}" textColor="#{RRGGBB}" shadeColor="#{RRGGBB}" borderFillIDRef="{id}">
    <hh:fontRef hangul="{0}" latin="{1}" hanja="{2}" japanese="{3}" other="{4}" symbol="{5}" user="{6}"/>
    <hh:ratio hangul="{0}" latin="{1}" .../>
    <hh:spacing hangul="{0}" .../>
    <hh:relSz hangul="{0}" .../>
    <hh:offset hangul="{0}" .../>
    <hh:bold/>           <!-- bold==true인 경우만 -->
    <hh:italic/>         <!-- italic==true인 경우만 -->
    <hh:underline type="BOTTOM" shape="SOLID" color="#000000"/>  <!-- 있는 경우만 -->
    <hh:strikeout shape="SOLID" color="#000000"/>                <!-- strikethrough==true인 경우만 -->
  </hh:charPr>
  ...
</hh:charProperties>
```

**ColorRef → XML**: `#RRGGBB` 형식 (파서 `parse_color()` 역방향).  
**underline_shape 역매핑**: `0→"SOLID"`, `1→"DASH"`, `2→"DOT"`, ... (파서 파악 완료).

### 단위 테스트

- `bold=true` → `<hh:bold/>` 존재 확인
- `underline_type=Bottom` → `<hh:underline type="BOTTOM"...>` 존재 확인
- charPr 라운드트립: parse → serialize → parse → IR 동등성 확인

---

## Stage 3: paraPr + tabPr + borderFills 직렬화

**파일**: `src/serializer/hwpx/header.rs`

### 구현 내용

#### 3-1. borderFills

```xml
<hh:borderFills>
  <hh:borderFill id="{i}">
    <hh:leftBorder type="NONE" width="0.12mm" color="#000000"/>
    <hh:rightBorder .../>
    <hh:topBorder .../>
    <hh:bottomBorder .../>
    <hh:diagonal type="0" .../>
    <!-- solid fill의 경우: -->
    <hh:fillBrush><hh:winBrush faceColor="#FFFFFF" hatchColor="#000000" alpha="0"/></hh:fillBrush>
  </hh:borderFill>
</hh:borderFills>
```

border width 역매핑: 5단계 (`"0.12mm"`, `"0.25mm"`, `"0.50mm"`, `"1.00mm"`, `"1.50mm"`, `"2.00mm"`, `"3.00mm"`, `"4.00mm"`, `"5.00mm"`).

#### 3-2. tabPr

```xml
<hh:tabProperties>
  <hh:tabPr id="{i}" autoTabLeft="{0|1}" autoTabRight="{0|1}">
    <hh:customTab pos="{pos}" type="{LEFT|CENTER|RIGHT|DECIMAL}" leader="{NONE|PERIOD|...}"/>
    ...
  </hh:tabPr>
</hh:tabProperties>
```

#### 3-3. paraPr (HwpUnitChar case 방식)

파서가 `<hh:switch><hh:case unit="HwpUnitChar">` 구조를 읽으므로 직렬화도 동일 구조:

```xml
<hh:paraProperties>
  <hh:paraPr id="{i}" tabPrIDRef="{id}" condense="0" fontLineHeight="0" ...>
    <hh:align horizontal="{JUSTIFY|LEFT|...}" vertical="BASELINE"/>
    <hh:switch>
      <hh:case unit="HwpUnitChar">
        <hh:margin left="{margin_left}" right="{margin_right}" prev="{spacing_before}" next="{spacing_after}" indent="{indent}"/>
        <hh:lineSpacing type="{PERCENT|FIXED|...}" value="{line_spacing}"/>
      </hh:case>
    </hh:switch>
  </hh:paraPr>
</hh:paraProperties>
```

`line_spacing` 역변환: FIXED/SpaceOnly/Minimum은 `value * 2` 했으므로 `/2` 복원.

### 단위 테스트

- borderFill 라운드트립 확인
- paraPr margin/lineSpacing 라운드트립 확인
- tabPr autoTabLeft/Right 확인

---

## Stage 4: styles + numberings + section.xml ID 연동

### 4-1. styles 직렬화

```xml
<hh:styles>
  <hh:style id="{i}" name="{local_name}" engName="{english_name}"
            type="{PARA|CHAR}" paraPrIDRef="{id}" charPrIDRef="{id}"
            nextStyleIDRef="{id}" langID="1042" lockForm="0"/>
</hh:styles>
```

### 4-2. numberings 직렬화

```xml
<hh:numberings>
  <hh:numbering id="{i}" start="{start}">
    <hh:paraHead id="{j}" start="{start}" level="{level}" .../>
  </hh:numbering>
</hh:numberings>
```

(bullets도 동일 패턴 — `<hh:bullets>`)

### 4-3. section.xml ID 동적 연동

**파일**: `src/serializer/hwpx/section.rs`

현재 `paraPrIDRef="0"` `charPrIDRef="0"` `styleIDRef="0"` 하드코딩 → `Paragraph` IR의 ID로 교체:

```rust
// 문단별:
write!(..., r#"paraPrIDRef="{}" styleIDRef="{}""#, para.para_shape_id, para.style_id)?;

// run별 (첫 char_shape):
let char_pr_id = para.char_shapes.first()
    .map(|cs| cs.char_shape_id)
    .unwrap_or(0);
write!(..., r#"charPrIDRef="{}""#, char_pr_id)?;
```

(run 분기는 이번 이슈 범위 밖 — 단일 run per paragraph 유지)

### 단위 테스트

- styles 라운드트립: name / paraPrIDRef / charPrIDRef 보존 확인
- section.xml의 paraPrIDRef / charPrIDRef / styleIDRef 값이 Paragraph IR 값과 일치 확인
- 실문서 라운드트립: `cargo run --example hwpx_roundtrip` — 글꼴명/스타일 보존 확인

---

## 공통 헬퍼

`src/serializer/hwpx/utils.rs`에 추가:
- `color_to_hex(c: ColorRef) -> String` — `#RRGGBB` 변환
- `underline_shape_to_str(shape: u8) -> &'static str`
- `border_width_to_str(w: u8) -> &'static str`
- `line_spacing_type_to_str(t: LineSpacingType) -> &'static str`
- `alignment_to_str(a: Alignment) -> &'static str`

---

## 예상 diff 규모

| 파일 | Stage | 신규/수정 LOC |
|------|-------|-------------|
| `src/serializer/hwpx/header.rs` | 1~4 | ~600 신규 |
| `src/serializer/hwpx/section.rs` | 4 | ~30 수정 |
| `src/serializer/hwpx/utils.rs` | 1~3 | ~60 신규 |
| tests (header.rs mod) | 1~4 | ~200 |
| **합계** | | **~890** |

---

## 검증 계획

```bash
cargo fmt --check
cargo clippy --all-targets
cargo test                          # 전체
cargo run --example hwpx_roundtrip  # 실문서 글꼴/스타일 보존
```

---

> **승인 요청**: 위 구현계획서를 검토 후 승인해 주시면 **Stage 1 구현**을 시작하겠습니다.
