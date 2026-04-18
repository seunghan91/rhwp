//! Contents/header.xml — DocInfo 리소스 테이블 직렬화
//!
//! Stage 1: fontfaces 동적 생성
//! Stage 2: charPr 동적 생성
//! Stage 3: borderFills / tabPr / paraPr 동적 생성
//! Stage 4+: styles / numberings + section.xml ID 연동

use crate::model::document::Document;
use crate::model::style::{
    Alignment, BorderFill, CharShape, Font, HeadType, LineSpacingType, Numbering, ParaShape,
    Style, TabDef, UnderlineType,
};
use super::SerializeError;
use super::utils::{
    alignment_to_str, border_line_type_to_str, border_width_to_str,
    color_ref_to_hex, line_shape_to_str, line_spacing_type_to_str, xml_escape,
};

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

fn write_border_fills(out: &mut String, border_fills: &[BorderFill]) {
    if border_fills.is_empty() {
        // 최소 placeholder: ID 1(없음), ID 2(winBrush) — charPr 기본값이 id=2 참조
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
        return;
    }

    out.push_str(&format!(r##"<hh:borderFills itemCnt="{}">"##, border_fills.len()));
    for (i, bf) in border_fills.iter().enumerate() {
        let id = i + 1; // 1-based
        out.push_str(&format!(
            r##"<hh:borderFill id="{id}" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0">"##
        ));
        out.push_str(r##"<hh:slash type="NONE" Crooked="0" isCounter="0"/>"##);
        out.push_str(r##"<hh:backSlash type="NONE" Crooked="0" isCounter="0"/>"##);
        let dir_names = ["left", "right", "top", "bottom"];
        for (di, dir) in dir_names.iter().enumerate() {
            let b = &bf.borders[di];
            let btype = border_line_type_to_str(&b.line_type);
            let bwidth = border_width_to_str(b.width);
            let bcolor = color_ref_to_hex(b.color);
            out.push_str(&format!(
                r##"<hh:{dir}Border type="{btype}" width="{bwidth}" color="{bcolor}"/>"##
            ));
        }
        {
            let d = &bf.diagonal;
            let dtype = border_line_type_to_str(&crate::model::style::BorderLineType::Solid);
            let _ = dtype;
            let dwidth = border_width_to_str(d.width);
            let dcolor = color_ref_to_hex(d.color);
            out.push_str(&format!(
                r##"<hh:diagonal type="{}" width="{dwidth}" color="{dcolor}"/>"##,
                if d.diagonal_type == 0 { "NONE" } else { "SOLID" }
            ));
        }
        use crate::model::style::FillType;
        if let FillType::Solid = bf.fill.fill_type {
            if let Some(ref s) = bf.fill.solid {
                let face = color_ref_to_hex(s.background_color);
                let hatch = color_ref_to_hex(s.pattern_color);
                let alpha = bf.fill.alpha;
                out.push_str(&format!(
                    r##"<hc:fillBrush><hc:winBrush faceColor="{face}" hatchColor="{hatch}" alpha="{alpha}"/></hc:fillBrush>"##
                ));
            }
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

fn write_tab_properties(out: &mut String, tab_defs: &[TabDef]) {
    if tab_defs.is_empty() {
        out.push_str(r##"<hh:tabProperties itemCnt="1"><hh:tabPr id="0" autoTabLeft="0" autoTabRight="0"/></hh:tabProperties>"##);
        return;
    }
    out.push_str(&format!(r##"<hh:tabProperties itemCnt="{}">"##, tab_defs.len()));
    for (i, td) in tab_defs.iter().enumerate() {
        let left  = if td.auto_tab_left  { "1" } else { "0" };
        let right = if td.auto_tab_right { "1" } else { "0" };
        if td.tabs.is_empty() {
            out.push_str(&format!(
                r##"<hh:tabPr id="{i}" autoTabLeft="{left}" autoTabRight="{right}"/>"##
            ));
        } else {
            out.push_str(&format!(
                r##"<hh:tabPr id="{i}" autoTabLeft="{left}" autoTabRight="{right}">"##
            ));
            for item in &td.tabs {
                let tab_type = match item.tab_type {
                    1 => "RIGHT",
                    2 => "CENTER",
                    3 => "DECIMAL",
                    _ => "LEFT",
                };
                let leader = match item.fill_type {
                    1 => "SOLID",
                    2 => "DOT",
                    3 => "DASH",
                    4 => "DASH_DOT",
                    5 => "DASH_DOT_DOT",
                    6 => "LONG_DASH",
                    7 => "CIRCLE",
                    8 => "DOUBLE_LINE",
                    9 => "THIN_THICK",
                    10 => "THICK_THIN",
                    11 => "TRIM",
                    _ => "NONE",
                };
                out.push_str(&format!(
                    r##"<hh:tabItem pos="{}" type="{tab_type}" leader="{leader}"/>"##,
                    item.position
                ));
            }
            out.push_str("</hh:tabPr>");
        }
    }
    out.push_str("</hh:tabProperties>");
}

fn write_para_properties(out: &mut String, para_shapes: &[ParaShape]) {
    if para_shapes.is_empty() {
        out.push_str(r##"<hh:paraProperties itemCnt="1">"##);
        write_single_para_pr(out, 0, &ParaShape::default());
        out.push_str("</hh:paraProperties>");
        return;
    }
    out.push_str(&format!(r##"<hh:paraProperties itemCnt="{}">"##, para_shapes.len()));
    for (i, ps) in para_shapes.iter().enumerate() {
        write_single_para_pr(out, i, ps);
    }
    out.push_str("</hh:paraProperties>");
}

fn write_single_para_pr(out: &mut String, id: usize, ps: &ParaShape) {
    out.push_str(&format!(
        r##"<hh:paraPr id="{id}" tabPrIDRef="{}" condense="0" fontLineHeight="0" snapToGrid="1" suppressLineNumbers="0" checked="0">"##,
        ps.tab_def_id
    ));

    let align = alignment_to_str(ps.alignment);
    out.push_str(&format!(r##"<hh:align horizontal="{align}" vertical="BASELINE"/>"##));

    let head_type_str = match ps.head_type {
        HeadType::Outline => "OUTLINE",
        HeadType::Number  => "NUMBER",
        HeadType::Bullet  => "BULLET",
        HeadType::None    => "NONE",
    };
    out.push_str(&format!(
        r##"<hh:heading type="{head_type_str}" idRef="{}" level="{}"/>"##,
        ps.numbering_id, ps.para_level
    ));

    let widow_orphan   = (ps.attr2 >> 5) & 1;
    let keep_with_next = (ps.attr2 >> 6) & 1;
    let keep_lines     = (ps.attr2 >> 7) & 1;
    let page_break     = (ps.attr2 >> 8) & 1;
    out.push_str(&format!(
        r##"<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" widowOrphan="{widow_orphan}" keepWithNext="{keep_with_next}" keepLines="{keep_lines}" pageBreakBefore="{page_break}" lineWrap="BREAK"/>"##
    ));

    let e_asian_eng = (ps.attr1 >> 20) & 1;
    let e_asian_num = (ps.attr1 >> 21) & 1;
    out.push_str(&format!(
        r##"<hh:autoSpacing eAsianEng="{e_asian_eng}" eAsianNum="{e_asian_num}"/>"##
    ));

    out.push_str(&format!(
        r##"<hh:margin left="{}" right="{}" indent="{}" prev="{}" next="{}"/>"##,
        ps.margin_left, ps.margin_right, ps.indent, ps.spacing_before, ps.spacing_after
    ));

    let ls_type = line_spacing_type_to_str(ps.line_spacing_type);
    out.push_str(&format!(
        r##"<hh:lineSpacing type="{ls_type}" value="{}"/>"##,
        ps.line_spacing
    ));

    out.push_str(&format!(
        r##"<hh:border borderFillIDRef="{}" offsetLeft="{}" offsetRight="{}" offsetTop="{}" offsetBottom="{}" connect="0" ignoreMargin="0"/>"##,
        ps.border_fill_id,
        ps.border_spacing[0], ps.border_spacing[1],
        ps.border_spacing[2], ps.border_spacing[3],
    ));

    out.push_str("</hh:paraPr>");
}

fn write_styles(out: &mut String, styles: &[Style]) {
    if styles.is_empty() {
        out.push_str(r##"<hh:styles itemCnt="1">"##);
        out.push_str(r##"<hh:style id="0" type="PARA" name="바탕글" engName="Normal" paraPrIDRef="0" charPrIDRef="0" nextStyleIDRef="0" langID="1042" lockForm="0"/>"##);
        out.push_str("</hh:styles>");
        return;
    }
    out.push_str(&format!(r##"<hh:styles itemCnt="{}">"##, styles.len()));
    for (i, s) in styles.iter().enumerate() {
        let stype = if s.style_type == 1 { "CHAR" } else { "PARA" };
        let name = xml_escape(&s.local_name);
        let engname = xml_escape(&s.english_name);
        out.push_str(&format!(
            r##"<hh:style id="{i}" type="{stype}" name="{name}" engName="{engname}" paraPrIDRef="{}" charPrIDRef="{}" nextStyleIDRef="{}" langID="1042" lockForm="0"/>"##,
            s.para_shape_id, s.char_shape_id, s.next_style_id
        ));
    }
    out.push_str("</hh:styles>");
}

fn write_numberings(out: &mut String, numberings: &[Numbering]) {
    if numberings.is_empty() {
        out.push_str(r##"<hh:numberings itemCnt="0"/>"##);
        return;
    }
    out.push_str(&format!(r##"<hh:numberings itemCnt="{}">"##, numberings.len()));
    for (i, num) in numberings.iter().enumerate() {
        out.push_str(&format!(r##"<hh:numbering id="{}" start="{}">"##, i + 1, num.start_number));
        for level in 0..7usize {
            let head = &num.heads[level];
            let fmt = num_format_to_str(head.number_format);
            let text = xml_escape(&num.level_formats[level]);
            let start = num.level_start_numbers[level];
            out.push_str(&format!(
                r##"<hh:paraHead level="{}" start="{}" text="{text}" numFormat="{fmt}" charPrIDRef="{}"/>"##,
                level + 1, start, head.char_shape_id
            ));
        }
        out.push_str("</hh:numbering>");
    }
    out.push_str("</hh:numberings>");
}

fn num_format_to_str(fmt: u8) -> &'static str {
    match fmt {
        1  => "CIRCLED_DIGIT",
        2  => "HANGUL_LETTER",
        3  => "HANGUL_NUMBER",
        4  => "HANGUL_CIRCLED_NUMBER",
        5  => "ROMAN_CAPITAL",
        6  => "ROMAN_SMALL",
        7  => "LATIN_CAPITAL",
        8  => "LATIN_SMALL",
        _  => "DIGIT",
    }
}

pub fn write_header(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let mut out = String::with_capacity(8192);

    out.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#);
    out.push_str(&format!("<hh:head {}>", HEAD_NS));

    out.push_str(r#"<hh:beginNum page="1" footnote="1" endnote="1" pic="1" tbl="1" equation="1"/>"#);

    out.push_str("<hh:refList>");
    write_fontfaces(&mut out, &doc.doc_info.font_faces);
    write_border_fills(&mut out, &doc.doc_info.border_fills);
    write_char_properties(&mut out, &doc.doc_info.char_shapes);
    write_tab_properties(&mut out, &doc.doc_info.tab_defs);
    write_numberings(&mut out, &doc.doc_info.numberings);
    write_para_properties(&mut out, &doc.doc_info.para_shapes);
    write_styles(&mut out, &doc.doc_info.styles);
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

    // ─── Stage 3: borderFills / tabPr / paraPr ───

    use crate::model::style::{
        Alignment, BorderFill, BorderLine, BorderLineType, Fill, FillType,
        LineSpacingType, ParaShape, SolidFill, TabDef, TabItem,
    };

    #[test]
    fn empty_border_fills_emits_placeholder() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"<hh:borderFills itemCnt="2">"##), "placeholder missing: {xml}");
    }

    #[test]
    fn border_fill_roundtrip() {
        let mut bf = BorderFill::default();
        bf.borders[0] = BorderLine { line_type: BorderLineType::Solid, width: 2, color: 0x000000FF }; // 빨강
        bf.fill.fill_type = FillType::Solid;
        bf.fill.solid = Some(SolidFill { background_color: 0x00FF0000, pattern_color: 0xFFFFFFFF, ..Default::default() });

        let mut doc = Document::default();
        doc.doc_info.border_fills.push(bf);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.border_fills.len(), 1);
        let p = &parsed.border_fills[0];
        assert_eq!(p.borders[0].line_type, BorderLineType::Solid);
        // 색상 역매핑: 빨강 0x000000FF → #FF0000 → parse_color → 0x000000FF
        assert_eq!(p.borders[0].color, 0x000000FF);
        assert_eq!(p.fill.fill_type, FillType::Solid);
        assert!(p.fill.solid.is_some());
        assert_eq!(p.fill.solid.unwrap().background_color, 0x00FF0000);
    }

    #[test]
    fn empty_tab_defs_emits_placeholder() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"<hh:tabProperties itemCnt="1">"##), "tab placeholder missing");
    }

    #[test]
    fn tab_def_auto_tab_roundtrip() {
        let mut doc = Document::default();
        doc.doc_info.tab_defs.push(TabDef { auto_tab_left: true, auto_tab_right: false, ..Default::default() });
        doc.doc_info.tab_defs.push(TabDef { auto_tab_left: false, auto_tab_right: true, ..Default::default() });

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.tab_defs.len(), 2);
        assert_eq!(parsed.tab_defs[0].auto_tab_left, true);
        assert_eq!(parsed.tab_defs[0].auto_tab_right, false);
        assert_eq!(parsed.tab_defs[1].auto_tab_left, false);
        assert_eq!(parsed.tab_defs[1].auto_tab_right, true);
    }

    #[test]
    fn tab_items_roundtrip() {
        let mut td = TabDef::default();
        td.tabs.push(TabItem { position: 2000, tab_type: 2, fill_type: 1 }); // CENTER, SOLID
        let mut doc = Document::default();
        doc.doc_info.tab_defs.push(td);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.tab_defs[0].tabs.len(), 1);
        // 직접 tabItem: 파서가 position 그대로 저장
        assert_eq!(parsed.tab_defs[0].tabs[0].position, 2000);
        assert_eq!(parsed.tab_defs[0].tabs[0].tab_type, 2); // CENTER
        assert_eq!(parsed.tab_defs[0].tabs[0].fill_type, 1); // SOLID
    }

    #[test]
    fn empty_para_shapes_emits_default_parappr() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"<hh:paraProperties itemCnt="1">"##), "paraPr placeholder missing");
    }

    // ─── Stage 4: styles / numberings ───

    use crate::model::style::{Numbering, NumberingHead, Style};

    #[test]
    fn empty_styles_emits_placeholder() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"<hh:styles itemCnt="1">"##), "styles placeholder missing: {xml}");
        assert!(xml.contains(r#"name="바탕글""#), "바탕글 missing");
    }

    #[test]
    fn style_roundtrip() {
        let mut doc = Document::default();
        doc.doc_info.styles.push(Style {
            local_name: "바탕글".to_string(),
            english_name: "Normal".to_string(),
            style_type: 0,
            next_style_id: 0,
            para_shape_id: 2,
            char_shape_id: 1,
            ..Default::default()
        });
        doc.doc_info.styles.push(Style {
            local_name: "본문".to_string(),
            english_name: "Body".to_string(),
            style_type: 0,
            next_style_id: 1,
            para_shape_id: 3,
            char_shape_id: 2,
            ..Default::default()
        });

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.styles.len(), 2);
        assert_eq!(parsed.styles[0].local_name, "바탕글");
        assert_eq!(parsed.styles[0].para_shape_id, 2);
        assert_eq!(parsed.styles[0].char_shape_id, 1);
        assert_eq!(parsed.styles[1].local_name, "본문");
        assert_eq!(parsed.styles[1].next_style_id, 1);
    }

    #[test]
    fn empty_numberings_emits_empty_tag() {
        let doc = Document::default();
        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        assert!(xml.contains(r##"<hh:numberings itemCnt="0"/>"##), "empty numberings missing: {xml}");
    }

    #[test]
    fn numbering_roundtrip() {
        let mut num = Numbering::default();
        num.start_number = 1;
        num.heads[0] = NumberingHead { number_format: 0, char_shape_id: 1, ..Default::default() };
        num.level_formats[0] = "^1.".to_string();
        num.level_start_numbers[0] = 1;

        let mut doc = Document::default();
        doc.doc_info.numberings.push(num);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.numberings.len(), 1);
        assert_eq!(parsed.numberings[0].start_number, 1);
        assert_eq!(parsed.numberings[0].level_start_numbers[0], 1);
    }

    #[test]
    fn para_shape_roundtrip() {
        let ps = ParaShape {
            tab_def_id: 1,
            alignment: Alignment::Left,
            margin_left: 3000,
            margin_right: 0,
            indent: -500,
            spacing_before: 400,
            spacing_after: 200,
            line_spacing: 160,
            line_spacing_type: LineSpacingType::Percent,
            border_fill_id: 2,
            ..Default::default()
        };
        let mut doc = Document::default();
        doc.doc_info.para_shapes.push(ps);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let xml = extract_header_xml(&bytes);
        let (parsed, _) = parse_hwpx_header(&xml).expect("parse header");

        assert_eq!(parsed.para_shapes.len(), 1);
        let p = &parsed.para_shapes[0];
        assert_eq!(p.tab_def_id, 1);
        assert_eq!(p.alignment, Alignment::Left);
        assert_eq!(p.margin_left, 3000);
        assert_eq!(p.indent, -500);
        assert_eq!(p.spacing_before, 400);
        assert_eq!(p.spacing_after, 200);
        assert_eq!(p.line_spacing, 160);
        assert_eq!(p.line_spacing_type, LineSpacingType::Percent);
        assert_eq!(p.border_fill_id, 2);
    }
}
