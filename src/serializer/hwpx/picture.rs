//! Contents/section{N}.xml — 그림(Picture) 직렬화
//! Stage 4 (#186): Control::Picture → <hp:pic>

use crate::model::image::{ImageAttr, ImageEffect, Picture};
use crate::model::shape::{HorzAlign, HorzRelTo, SizeCriterion, TextWrap, VertAlign, VertRelTo};

/// `<hp:pic>` 직렬화. 결과를 `out`에 append한다.
pub fn write_picture(out: &mut String, pic: &Picture) {
    let text_wrap = text_wrap_str(pic.common.text_wrap);
    out.push_str(&format!(
        r#"<hp:pic id="0" zOrder="{}" numberingType="PICTURE" textWrap="{}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" instid="{}">"#,
        pic.common.z_order,
        text_wrap,
        pic.common.instance_id,
    ));

    let width_rel = size_criterion_str(pic.common.width_criterion);
    let height_rel = size_criterion_str(pic.common.height_criterion);
    out.push_str(&format!(
        r#"<hp:sz width="{}" widthRelTo="{}" height="{}" heightRelTo="{}" protect="0"/>"#,
        pic.common.width, width_rel, pic.common.height, height_rel,
    ));

    let treat_as_char = if pic.common.treat_as_char { 1 } else { 0 };
    let vert_rel = vert_rel_str(pic.common.vert_rel_to);
    let horz_rel = horz_rel_str(pic.common.horz_rel_to);
    let vert_align = vert_align_str(pic.common.vert_align);
    let horz_align = horz_align_str(pic.common.horz_align);
    out.push_str(&format!(
        r#"<hp:pos treatAsChar="{}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="{}" horzRelTo="{}" vertAlign="{}" horzAlign="{}" vertOffset="{}" horzOffset="{}"/>"#,
        treat_as_char, vert_rel, horz_rel, vert_align, horz_align,
        pic.common.vertical_offset, pic.common.horizontal_offset,
    ));

    out.push_str(&format!(
        r#"<hp:outMargin left="{}" right="{}" top="{}" bottom="{}"/>"#,
        pic.common.margin.left, pic.common.margin.right,
        pic.common.margin.top, pic.common.margin.bottom,
    ));
    out.push_str(&format!(
        r#"<hp:inMargin left="{}" right="{}" top="{}" bottom="{}"/>"#,
        pic.padding.left, pic.padding.right, pic.padding.top, pic.padding.bottom,
    ));
    out.push_str(&format!(
        r#"<hp:imgClip left="{}" right="{}" top="{}" bottom="{}"/>"#,
        pic.crop.left, pic.crop.right, pic.crop.top, pic.crop.bottom,
    ));

    out.push_str(&image_tag(&pic.image_attr));
    out.push_str("</hp:pic>");
}

fn image_tag(ia: &ImageAttr) -> String {
    let effect = match ia.effect {
        ImageEffect::RealPic => "REAL_PIC",
        ImageEffect::GrayScale => "GRAY_SCALE",
        ImageEffect::BlackWhite => "BLACK_WHITE",
        ImageEffect::Pattern8x8 => "PATTERN_8X8",
    };
    format!(
        r#"<hc:img binaryItemIDRef="image{}" bright="{}" contrast="{}" effect="{}"/>"#,
        ia.bin_data_id, ia.brightness, ia.contrast, effect,
    )
}

/// BinData ZIP 경로 생성: "BinData/image{id}.{ext}"
pub fn bin_data_path(id: u16, ext: &str) -> String {
    format!("BinData/image{}.{}", id, ext)
}

/// MIME type 추론
pub fn media_type(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "tiff" | "tif" => "image/tiff",
        "wmf" => "image/x-wmf",
        "emf" => "image/x-emf",
        _ => "application/octet-stream",
    }
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
