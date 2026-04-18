//! Contents/section{N}.xml — Section 본문 직렬화
//!
//! Stage 2.3: 다문단 + 소프트 라인브레이크 + 탭 (한컴 레퍼런스 ref_mixed.hwpx 기반)
//! Stage 1/#186: secPr/pagePr 동적화
//!
//! IR 매핑 관행:
//!   - `section.paragraphs` 여러 개 = 하드 문단 경계 (`<hp:p>` 여러 개)
//!   - `paragraph.text` 내 `\n` = 소프트 라인브레이크 (`<hp:lineBreak/>`, 같은 문단 내)
//!   - `paragraph.text` 내 `\t` = 탭 (`<hp:tab width=... leader="0" type="1"/>`)

use crate::model::document::{Document, Section};
use crate::model::page::{BindingMethod, PageDef};
use crate::model::paragraph::Paragraph;
use super::utils::xml_escape;
use super::SerializeError;

const EMPTY_SECTION_XML: &str = include_str!("templates/empty_section0.xml");
const TEXT_RUN_SLOT: &str = r#"<hp:run charPrIDRef="0"><hp:t/></hp:run>"#;
const LINESEG_SLOT_OPEN: &str = "<hp:linesegarray>";
const LINESEG_SLOT_CLOSE: &str = "</hp:linesegarray>";
const PARA_CLOSE: &str = "</hp:p></hs:sec>";

/// 레퍼런스 기준 줄 레이아웃 파라미터.
const VERT_STEP: u32 = 1600; // vertsize(1000) + spacing(600)
const LINE_FLAGS: u32 = 393216;
const HORZ_SIZE: u32 = 42520;
/// 탭 기본 폭 — SectionDef.default_tab_spacing 이 0이면 사용하는 폴백 값.
const TAB_FALLBACK_WIDTH: u32 = 8000;

pub fn write_section(
    section: &Section,
    _doc: &Document,
    _index: usize,
) -> Result<Vec<u8>, SerializeError> {
    let mut vert_cursor: u32 = 0;
    let tab_width = {
        let s = section.section_def.default_tab_spacing;
        if s > 0 { s as u32 } else { TAB_FALLBACK_WIDTH }
    };

    let first_para = section.paragraphs.first();

    // 첫 문단: TEXT_RUN_SLOT 전체를 생성된 run(s)으로 치환.
    let mut out = if let Some(p) = first_para {
        let (first_runs, first_linesegs, first_advance) =
            render_paragraph_runs(p, vert_cursor, tab_width);
        vert_cursor = first_advance;
        let o = EMPTY_SECTION_XML.replacen(TEXT_RUN_SLOT, &first_runs, 1);
        replace_first_linesegs(&o, &first_linesegs)
    } else {
        EMPTY_SECTION_XML.to_string()
    };

    // secPr pagePr 동적화
    substitute_page_def(&mut out, &section.section_def.page_def);
    if section.section_def.default_tab_spacing > 0 {
        out = out.replacen(
            r#"tabStop="8000""#,
            &format!(r#"tabStop="{}""#, section.section_def.default_tab_spacing),
            1,
        );
    }

    // 첫 문단 paraPrIDRef / styleIDRef 동적 연동
    // (charPrIDRef는 render_paragraph_runs 내에서 처리됨)
    if let Some(p) = first_para {
        out = out.replacen(
            r#"paraPrIDRef="0""#,
            &format!(r#"paraPrIDRef="{}""#, p.para_shape_id),
            1,
        );
        out = out.replacen(
            r#"styleIDRef="0""#,
            &format!(r#"styleIDRef="{}""#, p.style_id),
            1,
        );
    }

    // 추가 문단: `</hp:p></hs:sec>` 직전에 `<hp:p>` 요소를 삽입.
    if section.paragraphs.len() > 1 {
        let mut extra = String::new();
        for p in &section.paragraphs[1..] {
            let (runs, linesegs, advance) = render_paragraph_runs(p, vert_cursor, tab_width);
            vert_cursor = advance;
            extra.push_str(&format!(
                r#"<hp:p id="0" paraPrIDRef="{}" styleIDRef="{}" pageBreak="0" columnBreak="0" merged="0">"#,
                p.para_shape_id, p.style_id,
            ));
            extra.push_str(&runs);
            extra.push_str(r#"<hp:linesegarray>"#);
            extra.push_str(&linesegs);
            extra.push_str(r#"</hp:linesegarray></hp:p>"#);
        }
        out = out.replacen(PARA_CLOSE, &format!("</hp:p>{}</hs:sec>", extra), 1);
    }

    Ok(out.into_bytes())
}

/// 문단 하나를 (`<hp:run>…</hp:run>` XML, lineseg XML, 다음 vert_cursor)로 변환.
///
/// `para.char_shapes`가 복수이면 UTF-16 offset 경계에서 run을 분리한다.
fn render_paragraph_runs(
    para: &Paragraph,
    vert_start: u32,
    default_tab_width: u32,
) -> (String, String, u32) {
    let text = &para.text;
    let tab_ext = &para.tab_extended;
    let shapes = &para.char_shapes;

    let mut runs_xml = String::new();
    let mut linesegs = String::new();
    push_lineseg(&mut linesegs, 0, vert_start);

    let first_id = shapes.first().map(|s| s.char_shape_id).unwrap_or(0);
    let mut shape_idx: usize = 0;
    let mut current_id = first_id;
    let mut t_xml = format!(r#"<hp:run charPrIDRef="{current_id}"><hp:t>"#);
    let mut buf = String::new();
    let mut utf16_pos: u32 = 0;
    let mut lines_in_para: u32 = 0;
    let mut tab_count: usize = 0;

    for c in text.chars() {
        let u16_len = c.len_utf16() as u32;

        // run 경계 도달 시 현재 run 닫고 새 run 열기
        while shape_idx + 1 < shapes.len()
            && utf16_pos >= shapes[shape_idx + 1].start_pos
        {
            flush_buf(&mut t_xml, &mut buf);
            t_xml.push_str("</hp:t></hp:run>");
            runs_xml.push_str(&t_xml);
            shape_idx += 1;
            current_id = shapes[shape_idx].char_shape_id;
            t_xml = format!(r#"<hp:run charPrIDRef="{current_id}"><hp:t>"#);
        }

        match c {
            '\t' => {
                flush_buf(&mut t_xml, &mut buf);
                let w = tab_ext.get(tab_count).map(|e| e[0] as u32).filter(|&v| v > 0).unwrap_or(default_tab_width);
                let leader = tab_ext.get(tab_count).map(|e| e[1]).unwrap_or(0);
                let ttype = tab_ext.get(tab_count).map(|e| e[2]).unwrap_or(1);
                t_xml.push_str(&format!(
                    r#"<hp:tab width="{w}" leader="{leader}" type="{ttype}"/>"#
                ));
                tab_count += 1;
                utf16_pos += u16_len;
            }
            '\n' => {
                flush_buf(&mut t_xml, &mut buf);
                t_xml.push_str("<hp:lineBreak/>");
                utf16_pos += u16_len;
                lines_in_para += 1;
                push_lineseg(
                    &mut linesegs,
                    utf16_pos,
                    vert_start + lines_in_para * VERT_STEP,
                );
            }
            c if (c as u32) < 0x20 => { /* 기타 제어문자 무시 */ }
            c => {
                buf.push(c);
                utf16_pos += u16_len;
            }
        }
    }

    // 마지막 run 닫기
    flush_buf(&mut t_xml, &mut buf);
    t_xml.push_str("</hp:t></hp:run>");
    runs_xml.push_str(&t_xml);

    let vert_end = vert_start + (lines_in_para + 1) * VERT_STEP;
    (runs_xml, linesegs, vert_end)
}

fn flush_buf(t_xml: &mut String, buf: &mut String) {
    if !buf.is_empty() {
        t_xml.push_str(&xml_escape(buf));
        buf.clear();
    }
}

fn push_lineseg(out: &mut String, textpos: u32, vertpos: u32) {
    out.push_str(&format!(
        r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="1000" textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="{}" flags="{}"/>"#,
        textpos, vertpos, HORZ_SIZE, LINE_FLAGS,
    ));
}

/// SectionDef.page_def 값으로 템플릿의 pagePr 고정값을 치환한다.
/// 각 패턴은 템플릿 내 고유하므로 replacen(…, 1) 사용.
/// page_def 필드가 0이면 템플릿 기본값 유지.
fn substitute_page_def(out: &mut String, pd: &PageDef) {
    if pd.landscape {
        // 기본값이 이미 WIDELY이므로 false(NARROW)일 때만 치환
    } else {
        *out = out.replacen(r#"landscape="WIDELY""#, r#"landscape="NARROW""#, 1);
    }

    let gutter_type = match pd.binding {
        BindingMethod::TopFlip => "TOP_ONLY",
        _ => "LEFT_ONLY",
    };
    if gutter_type != "LEFT_ONLY" {
        *out = out.replacen(
            r#"gutterType="LEFT_ONLY""#,
            &format!(r#"gutterType="{gutter_type}""#),
            1,
        );
    }

    if pd.width > 0 {
        *out = out.replacen(r#"width="59528""#, &format!(r#"width="{}""#, pd.width), 1);
    }
    if pd.height > 0 {
        *out = out.replacen(r#"height="84186""#, &format!(r#"height="{}""#, pd.height), 1);
    }
    if pd.margin_header > 0 {
        *out = out.replacen(
            r#"header="4252" footer="4252""#,
            &format!(r#"header="{}" footer="{}""#, pd.margin_header, pd.margin_footer),
            1,
        );
    }
    if pd.margin_gutter != 0 {
        *out = out.replacen(
            r#"gutter="0""#,
            &format!(r#"gutter="{}""#, pd.margin_gutter),
            1,
        );
    }
    if pd.margin_left > 0 {
        *out = out.replacen(
            r#"left="8504" right="8504""#,
            &format!(r#"left="{}" right="{}""#, pd.margin_left, pd.margin_right),
            1,
        );
    }
    if pd.margin_top > 0 {
        *out = out.replacen(
            r#"top="5668" bottom="4252""#,
            &format!(r#"top="{}" bottom="{}""#, pd.margin_top, pd.margin_bottom),
            1,
        );
    }
}

fn replace_first_linesegs(xml: &str, new_inner: &str) -> String {
    let open = xml.find(LINESEG_SLOT_OPEN).expect("template has linesegarray");
    let inner_start = open + LINESEG_SLOT_OPEN.len();
    let close_rel = xml[inner_start..]
        .find(LINESEG_SLOT_CLOSE)
        .expect("template has closing linesegarray");
    let inner_end = inner_start + close_rel;
    let mut out = String::with_capacity(xml.len() + new_inner.len());
    out.push_str(&xml[..inner_start]);
    out.push_str(new_inner);
    out.push_str(&xml[inner_end..]);
    out
}
