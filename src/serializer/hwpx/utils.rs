//! HWPX 직렬화 공용 헬퍼 — XML escape / 공통 이벤트 쓰기

use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use super::SerializeError;

/// `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` 선언을 쓴다.
pub fn write_xml_decl<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), Some("yes"))))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 속성 없는 시작 태그
pub fn start_tag<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    w.write_event(Event::Start(BytesStart::new(name)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 속성 있는 시작 태그
pub fn start_tag_attrs<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
) -> Result<(), SerializeError> {
    let mut el = BytesStart::new(name);
    for (k, v) in attrs {
        el.push_attribute((*k, *v));
    }
    w.write_event(Event::Start(el))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 종료 태그
pub fn end_tag<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    w.write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 자기 닫힘 태그 (`<name a="..."/>`)
pub fn empty_tag<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
) -> Result<(), SerializeError> {
    let mut el = BytesStart::new(name);
    for (k, v) in attrs {
        el.push_attribute((*k, *v));
    }
    w.write_event(Event::Empty(el))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 텍스트 노드 (자동 이스케이프)
pub fn text<W: Write>(w: &mut Writer<W>, content: &str) -> Result<(), SerializeError> {
    w.write_event(Event::Text(BytesText::new(content)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// HWP ColorRef(0x00BBGGRR) → `#RRGGBB` 또는 `"none"` (0xFFFFFFFF)
pub fn color_ref_to_hex(c: u32) -> String {
    if c == 0xFFFF_FFFF {
        return "none".to_string();
    }
    let r = c & 0xFF;
    let g = (c >> 8) & 0xFF;
    let b = (c >> 16) & 0xFF;
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// 선 종류 코드(표 27) → XML 문자열 역매핑
pub fn line_shape_to_str(shape: u8) -> &'static str {
    match shape {
        0 => "SOLID",
        1 => "DASH",
        2 => "DOT",
        3 => "DASH_DOT",
        4 => "DASH_DOT_DOT",
        5 => "LONG_DASH",
        6 => "CIRCLE",
        7 => "DOUBLE_SLIM",
        8 => "SLIM_THICK",
        9 => "THICK_SLIM",
        10 => "SLIM_THICK_SLIM",
        11 => "WAVE",
        12 => "DOUBLE_WAVE",
        _ => "SOLID",
    }
}

/// BorderLineType → OWPML 문자열 역매핑
pub fn border_line_type_to_str(t: &crate::model::style::BorderLineType) -> &'static str {
    use crate::model::style::BorderLineType::*;
    match t {
        None              => "NONE",
        Solid             => "SOLID",
        Dash              => "DASH",
        Dot               => "DOT",
        DashDot           => "DASH_DOT",
        DashDotDot        => "DASH_DOT_DOT",
        LongDash          => "LONG_DASH",
        Circle            => "CIRCLE",
        Double            => "DOUBLE_SLIM",
        ThinThickDouble   => "SLIM_THICK",
        ThickThinDouble   => "THICK_SLIM",
        ThinThickThinTriple => "SLIM_THICK_SLIM",
        Wave              => "WAVE",
        DoubleWave        => "DOUBLE_WAVE",
        _                 => "SOLID",
    }
}

/// BorderLine 굵기 인덱스(u8) → "N.N mm" 문자열
pub fn border_width_to_str(w: u8) -> &'static str {
    match w {
        0 => "0.1 mm",
        1 => "0.3 mm",
        2 => "0.5 mm",
        3 => "1.0 mm",
        4 => "1.5 mm",
        _ => "2.0 mm",
    }
}

/// Alignment → OWPML 문자열
pub fn alignment_to_str(a: crate::model::style::Alignment) -> &'static str {
    use crate::model::style::Alignment::*;
    match a {
        Justify    => "JUSTIFY",
        Left       => "LEFT",
        Right      => "RIGHT",
        Center     => "CENTER",
        Distribute => "DISTRIBUTE",
        Split      => "JUSTIFY",
    }
}

/// LineSpacingType → OWPML 문자열
pub fn line_spacing_type_to_str(t: crate::model::style::LineSpacingType) -> &'static str {
    use crate::model::style::LineSpacingType::*;
    match t {
        Percent   => "PERCENT",
        Fixed     => "FIXED",
        SpaceOnly => "SPACEONLY",
        Minimum   => "MINIMUM",
    }
}

/// XML 속성·텍스트 이스케이프 (&, <, >, ", ')
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}
