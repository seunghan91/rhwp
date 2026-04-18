//! Contents/header.xml — DocInfo 리소스 테이블 직렬화
//!
//! Stage 1: fontfaces 동적 생성
//! Stage 2: charPr 동적 생성
//! Stage 3+: paraPr / styles 등 순차적으로 동적 생성

use crate::model::document::Document;
use crate::model::style::{CharShape, Font, UnderlineType};
use super::SerializeError;
use super::utils::{color_ref_to_hex, line_shape_to_str, xml_escape};

const LANG_NAMES: [&str; 7] = ["HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER"];

/// HEAD 네임스페이스 선언 (고정)
const HEAD_NS: &str = concat!(
    r#"xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app" "#,
    r#"xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" "#,
    r#"xmlns:hp10="http://www.hancom.co.kr/hwpml/2016/paragraph" "#,
    r#"xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" "#,
    r#"xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core" "#,
    r#"xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head" "#,
    r#"xmlns:hhs="http://www.hancom.co.kr/hwpml/2011/history" "#,
    r#"xmlns:hm="http://www.hancom.co.kr/hwpml/2011/master-page" "#,
    r#"xmlns:hpf="http://www.hancom.co.kr/schema/2011/hpf" "#,
    r#"xmlns:dc="http://purl.org/dc/elements/1.1/" "#,
    r#"xmlns:opf="http://www.idpf.org/2007/opf/" "#,
    r#"xmlns:ooxmlchart="http://www.hancom.co.kr/hwpml/2016/ooxmlchart" "#,
    r#"xmlns:epub="http://www.idpf.org/2007/ops" "#,
    r#"xmlns:config="urn:oasis:names:tc:opendocument:xmlns:config:1.0" "#,
    r#"version="1.2" secCnt="1""#,
);

fn border_fills_placeholder(out: &mut String) {
    out.push_str(r##"<hh:borderFills itemCnt="2">"##);
    for id in [1u32, 2] {
        out.push_str(&format!(
            r##"<hh:borderFill id="{id}" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0">"##
        ));
        out.push_str(r##"<hh:slash type="NONE" Crooked="0" isCounter="0"/>"##);
        out.push_str(r##"<hh:backSlash type="NONE" Crooked="0" isCounter="0"/>"##);
        for dir in ["left", "right", "top", "bottom"] {
            out.push_str(&format!(
                r##"<hh:{dir}Border type="NONE" width="0.1 mm" color="#000000"/>"##
            ));
        }
        out.push_str(r##"<hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/>"##);
        if id == 2 {
            out.push_str(r##"<hc:fillBrush><hc:winBrush faceColor="none" hatchColor="#999999" alpha="0"/></hc:fillBrush>"##);
        }
        out.push_str("</hh:borderFill>");
    }
    out.push_str("</hh:borderFills>");
}

fn write_char_properties(out: &mut String, char_shapes: &[CharShape]) {
    // char_shapes가 비어 있으면 최소 1개(기본값) 출력 — section.xml의 charPrIDRef="0" 참조 보호
    if char_shapes.is_empty() {
        out.push_str(r##"<hh:charProperties itemCnt="1">"##);
        write_single_char_pr(out, 0, &CharShape::default());
        out.push_str("</hh:charProperties>");
        return;
    }
    out.push_str(&format!(r##"<hh:charProperties itemCnt="{}">"##, char_shapes.len()));
    for (i, cs) in char_shapes.iter().enumerate() {
        write_single_char_pr(out, i, cs);
    }
    out.push_str("</hh:charProperties>");
}

fn write_single_char_pr(out: &mut String, id: usize, cs: &CharShape) {
    let text_color = color_ref_to_hex(cs.text_color);
    let shade_color = color_ref_to_hex(cs.shade_color);
    out.push_str(&format!(
        r##"<hh:charPr id="{id}" height="{}" textColor="{text_color}" shadeColor="{shade_color}" useFontSpace="0" useKerning="{}" symMark="NONE" borderFillIDRef="{}">"##,
        cs.base_size,
        if cs.kerning { 1 } else { 0 },
        cs.border_fill_id,
    ));
    out.push_str(&lang7_attr("hh:fontRef", &cs.font_ids));
    out.push_str(&lang7_attr_u8("hh:ratio", &cs.ratios));
    out.push_str(&lang7_attr_i8("hh:spacing", &cs.spacings));
    out.push_str(&lang7_attr_u8("hh:relSz", &cs.relative_sizes));
    out.push_str(&lang7_attr_i8("hh:offset", &cs.char_offsets));

    if cs.bold    { out.push_str("<hh:bold/>"); }
    if cs.italic  { out.push_str("<hh:italic/>"); }

    if cs.underline_type != UnderlineType::None {
        let utype = match cs.underline_type {
            UnderlineType::Bottom => "BOTTOM",
            UnderlineType::Top    => "TOP",
            UnderlineType::None   => unreachable!(),
        };
        let ushape = line_shape_to_str(cs.underline_shape);
        let ucolor = color_ref_to_hex(cs.underline_color);
        out.push_str(&format!(
            r##"<hh:underline type="{utype}" shape="{ushape}" color="{ucolor}"/>"##
        ));
    }

    if cs.strikethrough {
        let sshape = line_shape_to_str(cs.strike_shape);
        let scolor = color_ref_to_hex(cs.strike_color);
        out.push_str(&format!(
            r##"<hh:strikeout shape="{sshape}" color="{scolor}"/>"##
        ));
    }

    if cs.outline_type != 0 {
        let otype = match cs.outline_type {
            1 => "SOLID", 2 => "DASH", 3 => "DOT", _ => "NONE",
        };
        out.push_str(&format!(r##"<hh:outline type="{otype}"/>"##));
    }

    if cs.shadow_type != 0 {
        let stype = "DROP";
        let scolor = color_ref_to_hex(cs.shadow_color);
        out.push_str(&format!(r##"<hh:shadow type="{stype}" color="{scolor}"/>"##));
    }

    if cs.emboss     { out.push_str("<hh:emboss/>"); }
    if cs.engrave    { out.push_str("<hh:engrave/>"); }
    if cs.superscript { out.push_str("<hh:supscript/>"); }
    if cs.subscript  { out.push_str("<hh:subscript/>"); }

    out.push_str("</hh:charPr>");
}

fn lang7_attr(tag: &str, vals: &[u16; 7]) -> String {
    format!(
        r##"<{tag} hangul="{}" latin="{}" hanja="{}" japanese="{}" other="{}" symbol="{}" user="{}"/>"##,
        vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6]
    )
}

fn lang7_attr_u8(tag: &str, vals: &[u8; 7]) -> String {
    format!(
        r##"<{tag} hangul="{}" latin="{}" hanja="{}" japanese="{}" other="{}" symbol="{}" user="{}"/>"##,
        vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6]
    )
}

fn lang7_attr_i8(tag: &str, vals: &[i8; 7]) -> String {
    format!(
        r##"<{tag} hangul="{}" latin="{}" hanja="{}" japanese="{}" other="{}" symbol="{}" user="{}"/>"##,
        vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6]
    )
}

fn tab_properties_placeholder(out: &mut String) {
    out.push_str(r##"<hh:tabProperties itemCnt="1"><hh:tabPr id="0" autoTabLeft="0" autoTabRight="0"/></hh:tabProperties>"##);
}

fn para_properties_placeholder(out: &mut String) {
    out.push_str(r##"<hh:paraProperties itemCnt="1">"##);
    out.push_str(r##"<hh:paraPr id="0" tabPrIDRef="0" condense="0" fontLineHeight="0" snapToGrid="1" suppressLineNumbers="0" checked="0">"##);
    out.push_str(r##"<hh:align horizontal="JUSTIFY" vertical="BASELINE"/>"##);
    out.push_str(r##"<hh:heading type="NONE" idRef="0" level="0"/>"##);
    out.push_str(r##"<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" widowOrphan="0" keepWithNext="0" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/>"##);
    out.push_str(r##"<hh:autoSpacing eAsianEng="0" eAsianNum="0"/>"##);
    out.push_str(r##"<hh:margin><hc:intent value="0" unit="HWPUNIT"/><hc:left value="0" unit="HWPUNIT"/><hc:right value="0" unit="HWPUNIT"/><hc:prev value="0" unit="HWPUNIT"/><hc:next value="0" unit="HWPUNIT"/></hh:margin>"##);
    out.push_str(r##"<hh:lineSpacing type="PERCENT" value="160" unit="HWPUNIT"/>"##);
    out.push_str(r##"<hh:border borderFillIDRef="2" offsetLeft="0" offsetRight="0" offsetTop="0" offsetBottom="0" connect="0" ignoreMargin="0"/>"##);
    out.push_str("</hh:paraPr>");
    out.push_str("</hh:paraProperties>");
}

fn styles_placeholder(out: &mut String) {
    out.push_str(r##"<hh:styles itemCnt="1">"##);
    out.push_str(r##"<hh:style id="0" type="PARA" name="바탕글" engName="Normal" paraPrIDRef="0" charPrIDRef="0" nextStyleIDRef="0" langID="1042" lockForm="0"/>"##);
    out.push_str("</hh:styles>");
}

pub fn write_header(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let mut out = String::with_capacity(8192);

    out.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#);
    out.push_str(&format!("<hh:head {}>", HEAD_NS));

    out.push_str(r#"<hh:beginNum page="1" footnote="1" endnote="1" pic="1" tbl="1" equation="1"/>"#);

    out.push_str("<hh:refList>");
    write_fontfaces(&mut out, &doc.doc_info.font_faces);
    border_fills_placeholder(&mut out);
    write_char_properties(&mut out, &doc.doc_info.char_shapes);
    tab_properties_placeholder(&mut out);
    out.push_str(r##"<hh:numberings itemCnt="0"/>"##);
    para_properties_placeholder(&mut out);
    styles_placeholder(&mut out);
    out.push_str("</hh:refList>");

    out.push_str(r#"<hh:compatibleDocument targetProgram="HWP201X"><hh:layoutCompatibility/></hh:compatibleDocument>"#);
    out.push_str(r#"<hh:docOption><hh:linkinfo path="" pageInherit="0" footnoteInherit="0"/></hh:docOption>"#);
    out.push_str(r#"<hh:trackchageConfig flags="56"/>"#);

    out.push_str("</hh:head>");

    Ok(out.into_bytes())
}

fn write_fontfaces(out: &mut String, font_faces: &[Vec<Font>]) {
    out.push_str(r#"<hh:fontfaces itemCnt="7">"#);
    for (i, lang) in LANG_NAMES.iter().enumerate() {
        let empty: Vec<Font> = Vec::new();
        let fonts: &[Font] = if i < font_faces.len() { &font_faces[i] } else { &empty };
        out.push_str(&format!(r#"<hh:fontface lang="{}" fontCnt="{}">"#, lang, fonts.len()));
        for (j, font) in fonts.iter().enumerate() {
            out.push_str(&format!(
                r#"<hh:font id="{}" face="{}" type="TTF" isEmbedded="0">"#,
                j,
                xml_escape(&font.name)
            ));
            out.push_str(r#"<hh:typeInfo familyType="FCAT_GOTHIC" weight="6" proportion="4" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" xHeight="1"/>"#);
            out.push_str("</hh:font>");
        }
        out.push_str("</hh:fontface>");
    }
    out.push_str("</hh:fontfaces>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Document;
    use crate::model::style::{CharShape, Font, UnderlineType};
    use crate::parser::hwpx::header::parse_hwpx_header;

    fn extract_header_xml(bytes: &[u8]) -> String {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut entry = archive.by_name("Contents/header.xml").expect("header.xml");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut entry, &mut xml).expect("read");
        xml
    }

    #[test]
    fn empty_doc_has_seven_fontfaces() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r#"<hh:fontfaces itemCnt="7">"#), "fontfaces missing: {xml}");
        for lang in ["HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER"] {
            assert!(
                xml.contains(&format!(r#"lang="{lang}""#)),
                "lang {lang} missing"
            );
        }
    }

    #[test]
    fn font_name_appears_in_fontfaces() {
        let mut doc = Document::default();
        doc.doc_info.font_faces = vec![Vec::new(); 7];
        doc.doc_info.font_faces[0].push(Font { name: "HY고딕".to_string(), ..Default::default() });
        doc.doc_info.font_faces[0].push(Font { name: "함초롬바탕".to_string(), ..Default::default() });

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r#"face="HY고딕""#), "HY고딕 missing: {xml}");
        assert!(xml.contains(r#"face="함초롬바탕""#), "함초롬바탕 missing");
        assert!(xml.contains(r#"fontCnt="2""#), "HANGUL fontCnt=2 missing");
    }

    #[test]
    fn font_face_name_xml_escaped() {
        let mut doc = Document::default();
        doc.doc_info.font_faces = vec![Vec::new(); 7];
        doc.doc_info.font_faces[0].push(Font { name: "A&B<C".to_string(), ..Default::default() });

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r#"face="A&amp;B&lt;C""#), "escape missing: {xml}");
    }

    #[test]
    fn font_faces_roundtrip() {
        let mut doc = Document::default();
        doc.doc_info.font_faces = vec![Vec::new(); 7];
        doc.doc_info.font_faces[0].push(Font { name: "함초롬돋움".to_string(), ..Default::default() });
        doc.doc_info.font_faces[1].push(Font { name: "Times New Roman".to_string(), ..Default::default() });

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed_info, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed_info.font_faces[0].len(), 1);
        assert_eq!(parsed_info.font_faces[0][0].name, "함초롬돋움");
        assert_eq!(parsed_info.font_faces[1].len(), 1);
        assert_eq!(parsed_info.font_faces[1][0].name, "Times New Roman");
        for i in 2..7 {
            assert!(parsed_info.font_faces[i].is_empty(), "group {i} should be empty");
        }
    }

    // ─── Stage 2: charPr ───

    fn make_charpr_doc(cs: CharShape) -> Document {
        let mut doc = Document::default();
        doc.doc_info.char_shapes.push(cs);
        doc
    }

    #[test]
    fn empty_charshapes_emits_default_charpr() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"<hh:charProperties itemCnt="1">"##), "default charPr missing: {xml}");
        assert!(xml.contains(r##"id="0""##));
    }

    #[test]
    fn charpr_bold_italic_emitted() {
        let cs = CharShape { bold: true, italic: true, base_size: 1000, ..Default::default() };
        let bytes = crate::serializer::hwpx::serialize_hwpx(&make_charpr_doc(cs)).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains("<hh:bold/>"), "bold missing");
        assert!(xml.contains("<hh:italic/>"), "italic missing");
    }

    #[test]
    fn charpr_no_bold_italic_when_false() {
        let cs = CharShape { bold: false, italic: false, ..Default::default() };
        let bytes = crate::serializer::hwpx::serialize_hwpx(&make_charpr_doc(cs)).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(!xml.contains("<hh:bold/>"), "bold should not appear");
        assert!(!xml.contains("<hh:italic/>"), "italic should not appear");
    }

    #[test]
    fn charpr_underline_bottom_emitted() {
        let cs = CharShape {
            underline_type: UnderlineType::Bottom,
            underline_shape: 0,
            underline_color: 0x00000000,
            ..Default::default()
        };
        let bytes = crate::serializer::hwpx::serialize_hwpx(&make_charpr_doc(cs)).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"type="BOTTOM""##), "underline BOTTOM missing: {xml}");
        assert!(xml.contains(r##"shape="SOLID""##), "shape missing");
    }

    #[test]
    fn charpr_strikeout_emitted() {
        let cs = CharShape { strikethrough: true, strike_shape: 0, strike_color: 0, ..Default::default() };
        let bytes = crate::serializer::hwpx::serialize_hwpx(&make_charpr_doc(cs)).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains("<hh:strikeout "), "strikeout missing: {xml}");
    }

    #[test]
    fn charpr_roundtrip() {
        let cs = CharShape {
            base_size: 1200,
            bold: true,
            font_ids: [1, 2, 0, 0, 0, 0, 0],
            ratios: [100, 90, 100, 100, 100, 100, 100],
            spacings: [0, -5, 0, 0, 0, 0, 0],
            relative_sizes: [100; 7],
            char_offsets: [0; 7],
            text_color: 0x000000FF, // 빨강 (#FF0000)
            shade_color: 0xFFFFFFFF,
            border_fill_id: 2,
            ..Default::default()
        };
        let mut doc = Document::default();
        doc.doc_info.char_shapes.push(cs.clone());

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.char_shapes.len(), 1);
        let p = &parsed.char_shapes[0];
        assert_eq!(p.base_size, 1200);
        assert_eq!(p.bold, true);
        assert_eq!(p.font_ids, [1, 2, 0, 0, 0, 0, 0]);
        assert_eq!(p.ratios, [100, 90, 100, 100, 100, 100, 100]);
        assert_eq!(p.spacings, [0, -5, 0, 0, 0, 0, 0]);
        assert_eq!(p.text_color, 0x000000FF);
        assert_eq!(p.shade_color, 0xFFFFFFFF);
        assert_eq!(p.border_fill_id, 2);
    }
}
