//! Contents/section{N}.xml — 표(Table) 직렬화
//! Stage 3 (#186): Control::Table → <hp:tbl>

use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::model::shape::{HorzAlign, HorzRelTo, SizeCriterion, TextWrap, VertAlign, VertRelTo};
use crate::model::paragraph::Paragraph;

/// `<hp:tbl>` 직렬화. 결과를 `out`에 append한다.
pub fn write_table(out: &mut String, tbl: &Table, default_tab_width: u32) {
    let page_break = match tbl.page_break {
        TablePageBreak::None => "NONE",
        TablePageBreak::CellBreak => "CELL",
        TablePageBreak::RowBreak => "ROW",
    };
    let text_wrap = text_wrap_str(tbl.common.text_wrap);

    out.push_str(&format!(
        r#"<hp:tbl id="{}" zOrder="{}" numberingType="TABLE" textWrap="{}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="{}" repeatHeader="{}" rowCnt="{}" colCnt="{}" cellSpacing="{}" borderFillIDRef="{}" noAdjust="0">"#,
        tbl.common.ctrl_id,
        tbl.common.z_order,
        text_wrap,
        page_break,
        if tbl.repeat_header { 1 } else { 0 },
        tbl.row_count,
        tbl.col_count,
        tbl.cell_spacing,
        tbl.border_fill_id,
    ));

    let width_rel = size_criterion_str(tbl.common.width_criterion);
    let height_rel = size_criterion_str(tbl.common.height_criterion);
    out.push_str(&format!(
        r#"<hp:sz width="{}" widthRelTo="{}" height="{}" heightRelTo="{}" protect="0"/>"#,
        tbl.common.width, width_rel, tbl.common.height, height_rel,
    ));

    let treat_as_char = if tbl.common.treat_as_char { 1 } else { 0 };
    let vert_rel = vert_rel_str(tbl.common.vert_rel_to);
    let horz_rel = horz_rel_str(tbl.common.horz_rel_to);
    let vert_align = vert_align_str(tbl.common.vert_align);
    let horz_align = horz_align_str(tbl.common.horz_align);
    out.push_str(&format!(
        r#"<hp:pos treatAsChar="{}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="{}" horzRelTo="{}" vertAlign="{}" horzAlign="{}" vertOffset="{}" horzOffset="{}"/>"#,
        treat_as_char, vert_rel, horz_rel, vert_align, horz_align,
        tbl.common.vertical_offset, tbl.common.horizontal_offset,
    ));

    out.push_str(&format!(
        r#"<hp:outMargin left="{}" right="{}" top="{}" bottom="{}"/>"#,
        tbl.outer_margin_left, tbl.outer_margin_right,
        tbl.outer_margin_top, tbl.outer_margin_bottom,
    ));

    out.push_str(&format!(
        r#"<hp:inMargin left="{}" right="{}" top="{}" bottom="{}"/>"#,
        tbl.padding.left, tbl.padding.right, tbl.padding.top, tbl.padding.bottom,
    ));

    for zone in &tbl.zones {
        out.push_str(&format!(
            r#"<hp:cellzone startColAddr="{}" endColAddr="{}" startRowAddr="{}" endRowAddr="{}" borderFillIDRef="{}"/>"#,
            zone.start_col, zone.end_col, zone.start_row, zone.end_row, zone.border_fill_id,
        ));
    }

    let mut current_row: i32 = -1;
    for cell in &tbl.cells {
        if cell.row as i32 != current_row {
            if current_row >= 0 {
                out.push_str("</hp:tr>");
            }
            out.push_str("<hp:tr>");
            current_row = cell.row as i32;
        }
        write_cell(out, cell, default_tab_width);
    }
    if current_row >= 0 {
        out.push_str("</hp:tr>");
    }

    out.push_str("</hp:tbl>");
}

fn write_cell(out: &mut String, cell: &Cell, default_tab_width: u32) {
    let vert_align = match cell.vertical_align {
        VerticalAlign::Top => "TOP",
        VerticalAlign::Center => "CENTER",
        VerticalAlign::Bottom => "BOTTOM",
    };
    let has_margin = if cell.apply_inner_margin { 1 } else { 0 };
    let is_header = if cell.is_header { 1 } else { 0 };
    let text_dir = if cell.text_direction == 0 { "HORIZONTAL" } else { "VERTICAL" };

    out.push_str(&format!(
        r#"<hp:tc name="" header="{}" hasMargin="{}" protect="0" editable="0" dirty="0" borderFillIDRef="{}">"#,
        is_header, has_margin, cell.border_fill_id,
    ));
    out.push_str(&format!(
        r#"<hp:subList id="" textDirection="{}" lineWrap="BREAK" vertAlign="{}" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">"#,
        text_dir, vert_align,
    ));

    if cell.paragraphs.is_empty() {
        out.push_str(
            r#"<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"/></hp:p>"#,
        );
    } else {
        for para in &cell.paragraphs {
            write_cell_para(out, para, default_tab_width);
        }
    }

    out.push_str("</hp:subList>");
    out.push_str(&format!(r#"<hp:cellAddr colAddr="{}" rowAddr="{}"/>"#, cell.col, cell.row));
    out.push_str(&format!(r#"<hp:cellSpan colSpan="{}" rowSpan="{}"/>"#, cell.col_span, cell.row_span));
    out.push_str(&format!(r#"<hp:cellSz width="{}" height="{}"/>"#, cell.width, cell.height));
    out.push_str(&format!(
        r#"<hp:cellMargin left="{}" right="{}" top="{}" bottom="{}"/>"#,
        cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
    ));
    out.push_str("</hp:tc>");
}

fn write_cell_para(out: &mut String, para: &Paragraph, default_tab_width: u32) {
    out.push_str(&format!(
        r#"<hp:p id="0" paraPrIDRef="{}" styleIDRef="{}" pageBreak="0" columnBreak="0" merged="0">"#,
        para.para_shape_id, para.style_id,
    ));

    if para.text.is_empty() && para.controls.is_empty() {
        let char_id = para.char_shapes.first().map(|s| s.char_shape_id).unwrap_or(0);
        out.push_str(&format!(r#"<hp:run charPrIDRef="{}"/>"#, char_id));
    } else {
        let (runs_xml, linesegs_xml, _) =
            crate::serializer::hwpx::section::render_paragraph_runs(para, 0, default_tab_width);
        out.push_str(&runs_xml);
        out.push_str("<hp:linesegarray>");
        out.push_str(&linesegs_xml);
        out.push_str("</hp:linesegarray>");
    }

    out.push_str("</hp:p>");
}

fn text_wrap_str(w: TextWrap) -> &'static str {
    match w {
        TextWrap::Square => "SQUARE",
        TextWrap::Tight => "TIGHT",
        TextWrap::Through => "THROUGH",
        TextWrap::TopAndBottom => "TOP_AND_BOTTOM",
        TextWrap::BehindText => "BEHIND_TEXT",
        TextWrap::InFrontOfText => "IN_FRONT_OF_TEXT",
    }
}

fn vert_rel_str(v: VertRelTo) -> &'static str {
    match v {
        VertRelTo::Paper => "PAPER",
        VertRelTo::Page => "PAGE",
        VertRelTo::Para => "PARA",
    }
}

fn horz_rel_str(h: HorzRelTo) -> &'static str {
    match h {
        HorzRelTo::Paper => "PAPER",
        HorzRelTo::Page => "PAGE",
        HorzRelTo::Column => "COLUMN",
        HorzRelTo::Para => "PARA",
    }
}

fn vert_align_str(v: VertAlign) -> &'static str {
    match v {
        VertAlign::Top => "TOP",
        VertAlign::Center => "CENTER",
        VertAlign::Bottom => "BOTTOM",
        VertAlign::Inside => "INSIDE",
        VertAlign::Outside => "OUTSIDE",
    }
}

fn horz_align_str(h: HorzAlign) -> &'static str {
    match h {
        HorzAlign::Left => "LEFT",
        HorzAlign::Center => "CENTER",
        HorzAlign::Right => "RIGHT",
        HorzAlign::Inside => "INSIDE",
        HorzAlign::Outside => "OUTSIDE",
    }
}

fn size_criterion_str(s: SizeCriterion) -> &'static str {
    match s {
        SizeCriterion::Paper => "PAPER",
        SizeCriterion::Page => "PAGE",
        SizeCriterion::Column => "COLUMN",
        SizeCriterion::Para => "PARA",
        SizeCriterion::Absolute => "ABSOLUTE",
    }
}
