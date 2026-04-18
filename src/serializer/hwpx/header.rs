//! Contents/header.xml — DocInfo 리소스 테이블 직렬화
//!
//! Stage 1: fontfaces 동적 생성 + 나머지 섹션 최소 placeholder
//! Stage 2+: charPr / paraPr / styles 등 순차적으로 동적 생성

use crate::model::document::Document;
use crate::model::style::Font;
use super::SerializeError;
use super::utils::xml_escape;

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

fn char_properties_placeholder(out: &mut String) {
    out.push_str(r##"<hh:charProperties itemCnt="1">"##);
    out.push_str(r##"<hh:charPr id="0" height="1000" textColor="#000000" shadeColor="none" useFontSpace="0" useKerning="0" symMark="NONE" borderFillIDRef="2">"##);
    let zero7 = r##"hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0""##;
    let one7  = r##"hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100""##;
    out.push_str(&format!(r##"<hh:fontRef {zero7}/>"##));
    out.push_str(&format!(r##"<hh:ratio {one7}/>"##));
    out.push_str(&format!(r##"<hh:spacing {zero7}/>"##));
    out.push_str(&format!(r##"<hh:relSz {one7}/>"##));
    out.push_str(&format!(r##"<hh:offset {zero7}/>"##));
    out.push_str("</hh:charPr>");
    out.push_str("</hh:charProperties>");
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
    char_properties_placeholder(&mut out);
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
    use crate::model::style::Font;
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
        // 나머지 5개 그룹은 빈 목록
        for i in 2..7 {
            assert!(parsed_info.font_faces[i].is_empty(), "group {i} should be empty");
        }
    }
}
