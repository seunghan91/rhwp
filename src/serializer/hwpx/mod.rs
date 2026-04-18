//! HWPX(ZIP+XML) 직렬화 모듈 — `parser::hwpx`의 역방향.
//!
//! ## 단계
//! - Stage 1 (현재): 한컴2020 호환 빈 HWPX 생성 (11개 필수 파일)
//! - Stage 2: 본문 문단·텍스트·lineSegArray IR 직렬화
//! - Stage 3: 표(Table)
//! - Stage 4: 그림(Picture) + BinData
//! - Stage 5: 라운드트립 테스트 + CLI

pub mod content;
pub mod header;
pub mod picture;
pub mod section;
pub mod static_assets;
pub mod table;
pub mod utils;
pub mod writer;

use crate::model::document::Document;

use super::SerializeError;
use writer::HwpxZipWriter;

/// Document IR을 HWPX(ZIP+XML) 바이트로 직렬화한다.
///
/// Stage 1: 한컴2020이 요구하는 11개 필수 파일을 모두 생성한다. 빈 Document의 경우
/// 보일러플레이트와 최소 골격(1개 섹션, 1개 문단)만 포함된다.
pub fn serialize_hwpx(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    use static_assets::*;

    let mut z = HwpxZipWriter::new();

    // 1. mimetype (반드시 최초 엔트리, STORED, extra field 없음)
    z.write_stored("mimetype", b"application/hwp+zip")?;

    // 2. version.xml
    z.write_deflated("version.xml", VERSION_XML.as_bytes())?;

    // 3. Contents/header.xml
    let header_xml = header::write_header(doc)?;
    z.write_deflated("Contents/header.xml", &header_xml)?;

    // 4. Contents/section{N}.xml — 실제 섹션만큼, 없으면 0개
    let section_hrefs: Vec<String> = (0..doc.sections.len())
        .map(|i| format!("Contents/section{}.xml", i))
        .collect();
    for (i, sec) in doc.sections.iter().enumerate() {
        let xml = section::write_section(sec, doc, i)?;
        z.write_deflated(&section_hrefs[i], &xml)?;
    }

    // 5. Preview/PrvText.txt + Preview/PrvImage.png
    z.write_deflated("Preview/PrvText.txt", PRV_TEXT)?;
    z.write_deflated("Preview/PrvImage.png", PRV_IMAGE_PNG)?;

    // 6. settings.xml
    z.write_deflated("settings.xml", SETTINGS_XML.as_bytes())?;

    // 7. META-INF/container.rdf
    z.write_deflated("META-INF/container.rdf", META_INF_CONTAINER_RDF.as_bytes())?;

    // 8. BinData ZIP 엔트리 수집 + 저장
    let bin_entries = collect_bin_data(doc, &mut z)?;

    // 9. Contents/content.hpf — BinData 없고 섹션 1개이면 템플릿 사용
    if doc.sections.len() == 1 && bin_entries.is_empty() {
        z.write_deflated("Contents/content.hpf", EMPTY_CONTENT_HPF.as_bytes())?;
    } else {
        let content_hpf = content::write_content_hpf(&section_hrefs, &bin_entries)?;
        z.write_deflated("Contents/content.hpf", &content_hpf)?;
    }

    // 10. META-INF/container.xml
    z.write_deflated("META-INF/container.xml", META_INF_CONTAINER_XML.as_bytes())?;

    // 11. META-INF/manifest.xml
    z.write_deflated("META-INF/manifest.xml", META_INF_MANIFEST_XML.as_bytes())?;

    z.finish()
}

/// 문서 전체를 순회하며 Picture 컨트롤을 찾아 BinData ZIP 엔트리를 기록하고
/// content.hpf manifest용 BinDataEntry 목록을 반환한다.
fn collect_bin_data(
    doc: &Document,
    z: &mut writer::HwpxZipWriter,
) -> Result<Vec<content::BinDataEntry>, SerializeError> {
    use crate::model::control::Control;

    let mut entries: Vec<content::BinDataEntry> = Vec::new();
    let mut written_ids: std::collections::HashSet<u16> = std::collections::HashSet::new();

    for sec in &doc.sections {
        collect_bin_data_in_paras(&sec.paragraphs, doc, z, &mut entries, &mut written_ids)?;
    }
    Ok(entries)
}

fn collect_bin_data_in_paras(
    paras: &[crate::model::paragraph::Paragraph],
    doc: &Document,
    z: &mut writer::HwpxZipWriter,
    entries: &mut Vec<content::BinDataEntry>,
    written_ids: &mut std::collections::HashSet<u16>,
) -> Result<(), SerializeError> {
    use crate::model::control::Control;

    for para in paras {
        for ctrl in &para.controls {
            match ctrl {
                Control::Picture(pic) => {
                    let id = pic.image_attr.bin_data_id;
                    if id > 0 && written_ids.insert(id) {
                        // BinDataContent에서 실제 데이터 탐색
                        if let Some(bc) = doc.bin_data_content.iter().find(|b| b.id == id) {
                            let ext = &bc.extension;
                            let path = picture::bin_data_path(id, ext);
                            let mime = picture::media_type(ext);
                            z.write_deflated(&path, &bc.data)?;
                            entries.push(content::BinDataEntry {
                                id: format!("image{}", id),
                                href: path,
                                media_type: mime.to_string(),
                            });
                        }
                    }
                }
                Control::Table(tbl) => {
                    // 셀 내 문단 재귀 처리
                    for cell in &tbl.cells {
                        collect_bin_data_in_paras(
                            &cell.paragraphs, doc, z, entries, written_ids,
                        )?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::hwpx::parse_hwpx;

    #[test]
    fn serialize_empty_doc_parses_back() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize empty");
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 0);
        assert!(parsed.bin_data_content.is_empty());
    }

    #[test]
    fn serialize_with_one_section_parses_back() {
        let mut doc = Document::default();
        doc.sections.push(crate::model::document::Section::default());
        let bytes = serialize_hwpx(&doc).expect("serialize one-section");
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 1);
    }

    #[test]
    fn serialize_text_paragraph_roundtrip() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "안녕 Hello 123".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize text");
        // 직렬화된 XML에 텍스트가 그대로 들어갔는지 ZIP에서 추출해 확인
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(
            xml.contains("<hp:t>안녕 Hello 123</hp:t>"),
            "text not injected into section0.xml"
        );

        // 라운드트립도 확인
        drop(sec0);
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 1);
        let p0 = &parsed.sections[0].paragraphs[0];
        assert!(
            p0.text.contains("안녕 Hello 123"),
            "text roundtrip failed: {:?}",
            p0.text
        );
    }

    #[test]
    fn tab_and_linebreak_emitted_inline() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A\tB\nC".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        // Stage 2.3 (ref_mixed 기반): 혼합 콘텐츠 + tab 속성 포함
        assert!(
            xml.contains(r#"<hp:t>A<hp:tab width="8000" leader="0" type="1"/>B<hp:lineBreak/>C</hp:t>"#),
            "mixed content not rendered: {}", xml
        );
    }

    #[test]
    fn linesegs_emitted_per_linebreak() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A\nB\nC".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");

        // 3줄(소프트) → lineseg 3개, textpos=0/2/4, vertpos=0/1600/3200
        let count = xml.matches("<hp:lineseg ").count();
        assert_eq!(count, 3, "expected 3 linesegs, got {}: {}", count, xml);
        assert!(xml.contains(r#"textpos="0" vertpos="0""#));
        assert!(xml.contains(r#"textpos="2" vertpos="1600""#));
        assert!(xml.contains(r#"textpos="4" vertpos="3200""#));
    }

    #[test]
    fn multi_paragraph_emits_multiple_hp_p() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        for t in ["첫째 줄", "둘째", "끝"] {
            let mut p = crate::model::paragraph::Paragraph::default();
            p.text = t.to_string();
            section.paragraphs.push(p);
        }
        doc.sections.push(section);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        let p_count = xml.matches("<hp:p ").count();
        assert_eq!(p_count, 3, "expected 3 <hp:p>, got {}", p_count);
        assert!(xml.contains("<hp:t>첫째 줄</hp:t>"));
        assert!(xml.contains("<hp:t>둘째</hp:t>"));
        assert!(xml.contains("<hp:t>끝</hp:t>"));
    }

    #[test]
    fn xml_escape_applied_to_section_text() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "a & b < c".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(xml.contains("a &amp; b &lt; c"), "escape missing: {}", xml);
    }

    #[test]
    fn mimetype_is_first_entry() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize");
        assert_eq!(&bytes[0..4], b"PK\x03\x04", "ZIP signature");
        let name_len = u16::from_le_bytes([bytes[26], bytes[27]]) as usize;
        let name = &bytes[30..30 + name_len];
        assert_eq!(name, b"mimetype");
    }

    #[test]
    fn mimetype_stored_not_deflated() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let method = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(method, 0, "mimetype must be STORED (method=0)");
    }

    #[test]
    fn hancom_required_files_present() {
        let mut doc = Document::default();
        doc.sections.push(crate::model::document::Section::default());
        let bytes = serialize_hwpx(&doc).expect("serialize");
        // ZIP 파일 목록에 한컴 필수 11개가 모두 있는지 확인
        let cursor = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        let required = [
            "mimetype",
            "version.xml",
            "Contents/header.xml",
            "Contents/section0.xml",
            "Contents/content.hpf",
            "Preview/PrvText.txt",
            "Preview/PrvImage.png",
            "settings.xml",
            "META-INF/container.xml",
            "META-INF/container.rdf",
            "META-INF/manifest.xml",
        ];
        for r in &required {
            assert!(
                names.iter().any(|n| n == r),
                "missing required file: {}",
                r
            );
        }
    }

    // ─── Stage 5/#186: 통합 검증 ───

    #[test]
    fn ref_table_roundtrip() {
        let bytes = std::fs::read("samples/hwpx/ref/ref_table.hwpx").expect("ref_table.hwpx");
        let doc = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse ref_table");
        let orig_tbl = doc.sections[0].paragraphs.iter()
            .flat_map(|p| p.controls.iter())
            .find_map(|c| if let crate::model::control::Control::Table(t) = c { Some(t.as_ref()) } else { None })
            .expect("no table in ref_table.hwpx");

        let rt_bytes = serialize_hwpx(&doc).expect("serialize");
        let rt_doc = crate::parser::hwpx::parse_hwpx(&rt_bytes).expect("parse rt");
        let rt_tbl = rt_doc.sections[0].paragraphs.iter()
            .flat_map(|p| p.controls.iter())
            .find_map(|c| if let crate::model::control::Control::Table(t) = c { Some(t.as_ref()) } else { None })
            .expect("no table in rt");
        assert_eq!(rt_tbl.row_count, orig_tbl.row_count, "row_count");
        assert_eq!(rt_tbl.col_count, orig_tbl.col_count, "col_count");
        assert_eq!(rt_tbl.cells.len(), orig_tbl.cells.len(), "cell count");
        assert_eq!(rt_tbl.border_fill_id, orig_tbl.border_fill_id, "borderFillIDRef");
    }

    #[test]
    fn ref_mixed_roundtrip() {
        let bytes = std::fs::read("samples/hwpx/ref/ref_mixed.hwpx").expect("ref_mixed.hwpx");
        let doc = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse ref_mixed");
        let orig_sec = &doc.sections[0];

        let rt_bytes = serialize_hwpx(&doc).expect("serialize");
        let rt_doc = crate::parser::hwpx::parse_hwpx(&rt_bytes).expect("parse rt");
        let rt_sec = &rt_doc.sections[0];
        assert_eq!(rt_sec.paragraphs.len(), orig_sec.paragraphs.len(), "paragraph count");
        for (i, (orig, rt)) in orig_sec.paragraphs.iter().zip(rt_sec.paragraphs.iter()).enumerate() {
            assert_eq!(rt.text, orig.text, "para[{i}] text");
        }
    }

    #[test]
    fn ref_text_roundtrip() {
        let bytes = std::fs::read("samples/hwpx/ref/ref_text.hwpx").expect("ref_text.hwpx");
        let doc = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse ref_text");
        let rt_bytes = serialize_hwpx(&doc).expect("serialize");
        let rt_doc = crate::parser::hwpx::parse_hwpx(&rt_bytes).expect("parse rt");
        assert_eq!(rt_doc.sections.len(), doc.sections.len(), "section count");
        for (i, (orig, rt)) in doc.sections[0].paragraphs.iter().zip(rt_doc.sections[0].paragraphs.iter()).enumerate() {
            assert_eq!(rt.text, orig.text, "para[{i}] text");
        }
    }

    // ─── Stage 4/#186: 그림 직렬화 + BinData ───

    fn make_picture_doc(bin_data_id: u16, ext: &str, data: Vec<u8>, width: u32, height: u32) -> Document {
        use crate::model::image::{Picture, ImageAttr};
        use crate::model::control::Control;
        use crate::model::bin_data::BinDataContent;
        let mut doc = Document::default();

        // BinDataContent 등록
        doc.bin_data_content.push(BinDataContent {
            id: bin_data_id,
            data,
            extension: ext.to_string(),
        });

        let mut sec = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        let mut pic = Picture::default();
        pic.common.width = width;
        pic.common.height = height;
        pic.image_attr = ImageAttr { bin_data_id, ..Default::default() };
        para.text = "\u{0002}".to_string();
        para.controls.push(Control::Picture(Box::new(pic)));
        sec.paragraphs.push(para);
        doc.sections.push(sec);
        doc
    }

    #[test]
    fn picture_bindata_entry_generated() {
        let png_bytes = vec![0x89u8, 0x50, 0x4e, 0x47]; // PNG magic
        let doc = make_picture_doc(1, "png", png_bytes, 5000, 3000);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).expect("zip");
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        assert!(names.iter().any(|n| n == "BinData/image1.png"), "BinData missing: {:?}", names);
    }

    #[test]
    fn picture_manifest_entry() {
        let png_bytes = vec![0x89u8, 0x50, 0x4e, 0x47];
        let doc = make_picture_doc(1, "png", png_bytes, 5000, 3000);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut hpf = archive.by_name("Contents/content.hpf").expect("content.hpf");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut hpf, &mut xml).expect("read");
        assert!(xml.contains("BinData/image1.png"), "manifest missing: {xml}");
        assert!(xml.contains("image/png"), "media-type missing");
    }

    #[test]
    fn picture_roundtrip_attr() {
        let png_bytes = vec![0x89u8, 0x50, 0x4e, 0x47];
        let doc = make_picture_doc(2, "png", png_bytes, 7200, 5400);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        assert!(xml.contains("<hp:pic "), "hp:pic missing");
        assert!(xml.contains(r#"binaryItemIDRef="image2""#), "binaryItemIDRef missing: {xml}");
        assert!(xml.contains(r#"width="7200""#), "width missing");
        assert!(xml.contains(r#"height="5400""#), "height missing");

        let parsed = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse back");
        let p0 = &parsed.sections[0].paragraphs[0];
        let pic_ctrl = p0.controls.iter().find_map(|c| {
            if let crate::model::control::Control::Picture(p) = c { Some(p.as_ref()) } else { None }
        }).expect("Picture control not found");
        assert_eq!(pic_ctrl.image_attr.bin_data_id, 2, "bin_data_id roundtrip");
        assert_eq!(pic_ctrl.common.width, 7200, "width roundtrip");
        assert_eq!(pic_ctrl.common.height, 5400, "height roundtrip");
    }

    // ─── Stage 3/#186: 표 직렬화 ───

    fn make_table_doc(row_count: u16, col_count: u16, border_fill_id: u16) -> Document {
        use crate::model::table::{Table, Cell};
        use crate::model::control::Control;
        let mut doc = Document::default();
        let mut sec = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();

        let mut tbl = Table::default();
        tbl.row_count = row_count;
        tbl.col_count = col_count;
        tbl.border_fill_id = border_fill_id;
        for r in 0..row_count {
            for c in 0..col_count {
                tbl.cells.push(Cell::new_empty(c, r, 5000, 1000, border_fill_id));
            }
        }
        para.text = "\u{0002}".to_string();
        para.controls.push(Control::Table(Box::new(tbl)));
        sec.paragraphs.push(para);
        doc.sections.push(sec);
        doc
    }

    fn find_table(para: &crate::model::paragraph::Paragraph) -> &crate::model::table::Table {
        para.controls.iter().find_map(|c| {
            if let crate::model::control::Control::Table(t) = c { Some(t.as_ref()) } else { None }
        }).expect("Table control not found")
    }

    #[test]
    fn empty_table_roundtrip() {
        let doc = make_table_doc(2, 3, 1);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let parsed = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 1);
        let p0 = &parsed.sections[0].paragraphs[0];
        let tbl = find_table(p0);
        assert_eq!(tbl.row_count, 2, "row_count");
        assert_eq!(tbl.col_count, 3, "col_count");
    }

    #[test]
    fn table_cell_text_roundtrip() {
        use crate::model::table::{Table, Cell};
        use crate::model::control::Control;
        let mut doc = Document::default();
        let mut sec = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();

        let mut tbl = Table::default();
        tbl.row_count = 1;
        tbl.col_count = 1;
        tbl.border_fill_id = 1;
        let mut cell = Cell::new_empty(0, 0, 5000, 1000, 1);
        cell.paragraphs.clear();
        let mut cp = crate::model::paragraph::Paragraph::default();
        cp.text = "셀 텍스트".to_string();
        cell.paragraphs.push(cp);
        tbl.cells.push(cell);

        para.text = "\u{0002}".to_string();
        para.controls.push(Control::Table(Box::new(tbl)));
        sec.paragraphs.push(para);
        doc.sections.push(sec);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        assert!(xml.contains("<hp:tbl "), "tbl missing: {}", &xml[..xml.len().min(500)]);
        assert!(xml.contains("셀 텍스트"), "cell text missing");

        let parsed = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse back");
        let p0 = &parsed.sections[0].paragraphs[0];
        let tbl = find_table(p0);
        assert_eq!(tbl.row_count, 1);
        let cell_text = tbl.cells[0].paragraphs.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("");
        assert!(cell_text.contains("셀 텍스트"), "cell text roundtrip: {:?}", cell_text);
    }

    #[test]
    fn table_borderfillidref() {
        let doc = make_table_doc(1, 1, 7);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        assert!(xml.contains(r#"borderFillIDRef="7""#), "borderFillIDRef missing: {}", &xml[..xml.len().min(500)]);
    }

    #[test]
    fn table_cellspan_roundtrip() {
        use crate::model::table::{Table, Cell};
        use crate::model::control::Control;
        let mut doc = Document::default();
        let mut sec = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();

        let mut tbl = Table::default();
        tbl.row_count = 2;
        tbl.col_count = 2;
        tbl.border_fill_id = 1;
        let mut merged = Cell::new_empty(0, 0, 10000, 1000, 1);
        merged.col_span = 2;
        tbl.cells.push(merged);
        tbl.cells.push(Cell::new_empty(0, 1, 5000, 1000, 1));
        tbl.cells.push(Cell::new_empty(1, 1, 5000, 1000, 1));
        para.text = "\u{0002}".to_string();
        para.controls.push(Control::Table(Box::new(tbl)));
        sec.paragraphs.push(para);
        doc.sections.push(sec);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let parsed = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse back");
        let p0 = &parsed.sections[0].paragraphs[0];
        let tbl = find_table(p0);
        assert_eq!(tbl.cells[0].col_span, 2, "colSpan");
        assert_eq!(tbl.cells[0].row_span, 1, "rowSpan");
    }

    // ─── Stage 1/#186: pagePr 동적화 ───

    fn extract_section0_xml(bytes: &[u8]) -> String {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut entry = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut entry, &mut xml).expect("read");
        xml
    }

    fn make_section_doc(pd: crate::model::page::PageDef) -> Document {
        let mut doc = Document::default();
        let mut sec = crate::model::document::Section::default();
        sec.section_def.page_def = pd;
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "test".to_string();
        sec.paragraphs.push(para);
        doc.sections.push(sec);
        doc
    }

    #[test]
    fn pagePr_dynamic_width_height() {
        use crate::model::page::PageDef;
        let mut pd = PageDef::default();
        pd.width = 42000;
        pd.height = 59528;
        pd.landscape = true;
        let bytes = serialize_hwpx(&make_section_doc(pd)).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        assert!(xml.contains(r#"width="42000""#), "width missing: {xml}");
        assert!(xml.contains(r#"height="59528""#), "height missing");
        assert!(xml.contains(r#"landscape="WIDELY""#), "landscape WIDELY missing");
    }

    #[test]
    fn pagePr_margins_dynamic() {
        use crate::model::page::PageDef;
        let pd = PageDef {
            width: 59528, height: 84186,
            margin_left: 9000, margin_right: 9000,
            margin_top: 6000, margin_bottom: 5000,
            margin_header: 4000, margin_footer: 4000,
            margin_gutter: 1000,
            landscape: true,
            ..Default::default()
        };
        let bytes = serialize_hwpx(&make_section_doc(pd)).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        assert!(xml.contains(r#"left="9000" right="9000""#), "lr missing: {xml}");
        assert!(xml.contains(r#"top="6000" bottom="5000""#), "tb missing");
        assert!(xml.contains(r#"header="4000" footer="4000""#), "header missing");
        assert!(xml.contains(r#"gutter="1000""#), "gutter missing");
    }

    // ─── Stage 2/#186: 다중 run 분할 ───

    #[test]
    fn single_charshape_single_run() {
        use crate::model::paragraph::CharShapeRef;
        let mut doc = Document::default();
        let mut sec = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "hello".to_string();
        para.char_shapes = vec![CharShapeRef { start_pos: 0, char_shape_id: 7 }];
        sec.paragraphs.push(para);
        doc.sections.push(sec);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        let run_count = xml.matches("<hp:run ").count();
        // 두 번째 run만 (첫 run은 secPr 포함 run)
        assert_eq!(run_count, 2, "expected 2 hp:run (secPr + text), got {run_count}: {xml}");
        assert!(xml.contains(r#"charPrIDRef="7""#), "charPrIDRef=7 missing: {xml}");
        assert!(xml.contains("<hp:t>hello</hp:t>"), "text missing: {xml}");
    }

    #[test]
    fn multi_run_splits_correctly() {
        use crate::model::paragraph::CharShapeRef;
        let mut doc = Document::default();
        let mut sec = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        // "AB CD" — A,B belong to shape 1 (pos 0), space belongs to shape 2 (pos 2), C,D shape 3 (pos 3)
        para.text = "AB CD".to_string();
        para.char_shapes = vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
            CharShapeRef { start_pos: 2, char_shape_id: 2 },
            CharShapeRef { start_pos: 3, char_shape_id: 3 },
        ];
        sec.paragraphs.push(para);
        doc.sections.push(sec);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        // 3 text runs + 1 secPr run = 4 total
        let run_count = xml.matches("<hp:run ").count();
        assert_eq!(run_count, 4, "expected 4 hp:run, got {run_count}: {xml}");
        assert!(xml.contains(r#"charPrIDRef="1"><hp:t>AB</hp:t>"#), "run1 missing: {xml}");
        assert!(xml.contains(r#"charPrIDRef="2"><hp:t> </hp:t>"#), "run2 missing: {xml}");
        assert!(xml.contains(r#"charPrIDRef="3"><hp:t>CD</hp:t>"#), "run3 missing: {xml}");
    }

    #[test]
    fn pagePr_landscape_narrow() {
        use crate::model::page::PageDef;
        let pd = PageDef { landscape: false, width: 59528, height: 84186, ..Default::default() };
        let bytes = serialize_hwpx(&make_section_doc(pd)).expect("serialize");
        let xml = extract_section0_xml(&bytes);
        assert!(xml.contains(r#"landscape="NARROW""#), "NARROW missing: {xml}");
    }
}
