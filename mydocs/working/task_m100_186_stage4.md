# Stage 4 완료 보고서 — Task M100 #186
# 그림 직렬화 + BinData ZIP 저장 + content.hpf 연동

## 완료 일자
2026-04-18

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/picture.rs` | 신규: write_picture, bin_data_path, media_type |
| `src/serializer/hwpx/section.rs` | Control::Picture dispatcher 추가 |
| `src/serializer/hwpx/mod.rs` | `pub mod picture;`, collect_bin_data, Stage 4 테스트 3개 |

## 구현 요약

### picture.rs (신규)

```rust
pub fn write_picture(out: &mut String, pic: &Picture)
```

출력 구조:
```xml
<hp:pic id="0" zOrder="0" numberingType="PICTURE" textWrap="SQUARE" ... instid="0">
  <hp:sz width="N" widthRelTo="ABSOLUTE" height="N" heightRelTo="ABSOLUTE" protect="0"/>
  <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="COLUMN" .../>
  <hp:outMargin left="N" right="N" top="N" bottom="N"/>
  <hp:inMargin left="N" right="N" top="N" bottom="N"/>
  <hp:imgClip left="N" right="N" top="N" bottom="N"/>
  <hc:img binaryItemIDRef="image{id}" bright="N" contrast="N" effect="REAL_PIC"/>
</hp:pic>
```

파싱 규칙: `binaryItemIDRef="image{N}"` → 숫자만 추출 → `bin_data_id = N`

### collect_bin_data (mod.rs)

```rust
fn collect_bin_data(doc, z) -> Result<Vec<BinDataEntry>, SerializeError>
```

- 문서 전체 문단+셀 재귀 순회
- `Control::Picture` 발견 시 `bin_data_id` → `doc.bin_data_content`에서 실제 바이트 탐색
- ZIP에 `BinData/image{id}.{ext}` 저장
- `content.hpf`용 `BinDataEntry { id: "image{id}", href, media_type }` 반환

### Control dispatcher 확장 (section.rs)

```rust
Control::Picture(pic) => {
    crate::serializer::hwpx::picture::write_picture(&mut ctrl_section, pic);
}
```

표와 동일한 구조: `ctrl_section`에 누적 → `<hp:t>` 이전에 출력

### content.hpf 연동

`serialize_hwpx`에서 `bin_entries` 존재 시 동적 content.hpf 생성:
```
기존: doc.sections.len()==1 && doc.bin_data_content.is_empty() → 템플릿 사용
변경: doc.sections.len()==1 && bin_entries.is_empty() → 템플릿 사용
```
bin_entries가 있으면 항상 `write_content_hpf` 호출.

## 테스트 결과

| 테스트 | 결과 |
|--------|------|
| `picture_bindata_entry_generated` | ✅ BinData/image1.png ZIP 엔트리 생성 |
| `picture_manifest_entry` | ✅ content.hpf에 image/png 항목 포함 |
| `picture_roundtrip_attr` | ✅ bin_data_id/width/height 라운드트립 |
| 기존 40개 | ✅ 회귀 없음 |
| **합계** | **43 passed** |
