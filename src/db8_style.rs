use crate::db8_document::{Db8Document, Db8Span, StyleSet};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Db8StyleConfig {
    pub page: PageStyle,
    pub body: BodyStyle,
    pub pocket: PocketStyle,
    pub hat: HatStyle,
    pub block: BlockStyle,
    pub tag: TagStyle,
    pub cite: CiteStyle,
    pub normal: NormalStyle,
    pub highlight: HighlightStyle,
    pub emphasis: EmphasisStyle,
    pub underline: UnderlineStyle,
    pub shrunk: ShrunkStyle,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageStyle {
    pub width: String,
    pub margin_left: String,
    pub margin_right: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyStyle {
    pub font_family: String,
    pub font_size_pt: f32,
    pub line_height: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PocketStyle {
    pub font_size_pt: f32,
    pub font_weight: u32,
    pub spacing_before_pt: f32,
    pub spacing_after_pt: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HatStyle {
    pub font_size_pt: f32,
    pub font_weight: u32,
    pub text_color: String,
    pub spacing_before_pt: f32,
    pub spacing_after_pt: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockStyle {
    pub spacing_after_pt: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagStyle {
    pub font_family: String,
    pub font_size_pt: f32,
    pub font_weight: u32,
    pub text_color: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiteStyle {
    pub font_size_pt: f32,
    pub italic: bool,
    pub text_color: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalStyle {
    pub font_size_pt: f32,
    pub text_color: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighlightStyle {
    pub background: String,
    pub text_color: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmphasisStyle {
    pub font_weight: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnderlineStyle {
    pub color: String,
    pub thickness_px: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShrunkStyle {
    pub font_size_scale: f32,
    pub line_height: f32,
}

impl Default for Db8StyleConfig {
    fn default() -> Self {
        Self {
            page: PageStyle {
                width: "8.5in".to_string(),
                margin_left: "1in".to_string(),
                margin_right: "1in".to_string(),
            },
            body: BodyStyle {
                font_family: "Calibri".to_string(),
                font_size_pt: 11.0,
                line_height: 1.15,
            },
            pocket: PocketStyle {
                font_size_pt: 26.0,
                font_weight: 700,
                spacing_before_pt: 12.0,
                spacing_after_pt: 8.0,
            },
            hat: HatStyle {
                font_size_pt: 22.0,
                font_weight: 700,
                text_color: "#111111".to_string(),
                spacing_before_pt: 8.0,
                spacing_after_pt: 4.0,
            },
            block: BlockStyle {
                spacing_after_pt: 10.0,
            },
            tag: TagStyle {
                font_family: "Calibri".to_string(),
                font_size_pt: 13.0,
                font_weight: 700,
                text_color: "#000000".to_string(),
            },
            cite: CiteStyle {
                font_size_pt: 13.0,
                italic: false,
                text_color: "#000000".to_string(),
            },
            normal: NormalStyle {
                font_size_pt: 11.0,
                text_color: "#000000".to_string(),
            },
            highlight: HighlightStyle {
                background: "#fff176".to_string(),
                text_color: "#000000".to_string(),
            },
            emphasis: EmphasisStyle { font_weight: 700 },
            underline: UnderlineStyle {
                color: "#000000".to_string(),
                thickness_px: 1,
            },
            shrunk: ShrunkStyle {
                font_size_scale: 0.82,
                line_height: 1.0,
            },
        }
    }
}

pub fn load_or_create_style_config(path: &Path) -> Db8StyleConfig {
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<Db8StyleConfig>(&raw).unwrap_or_else(|_| {
            let fallback = Db8StyleConfig::default();
            let _ = save_style_config(path, &fallback);
            fallback
        }),
        Err(_) => {
            let fallback = Db8StyleConfig::default();
            let _ = save_style_config(path, &fallback);
            fallback
        }
    }
}

pub fn save_style_config(path: &Path, config: &Db8StyleConfig) -> std::io::Result<()> {
    let raw = serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string());
    fs::write(path, raw)
}

pub fn style_config_to_css(config: &Db8StyleConfig) -> String {
    format!(
        ".db8-editor {{\n  width: {};\n  margin-left: {};\n  margin-right: {};\n  font-family: \"{}\";\n  font-size: {}pt;\n  line-height: {};\n}}\n\n.db8-pocket {{\n  font-size: {}pt;\n  font-weight: {};\n  margin-top: {}pt;\n  margin-bottom: {}pt;\n  border: 1px solid #000000;\n  padding: 2pt 6pt;\n  text-align: center;\n}}\n\n.db8-hat {{\n  font-size: {}pt;\n  font-weight: {};\n  color: {};\n  margin-top: {}pt;\n  margin-bottom: {}pt;\n  text-decoration-line: underline;\n  text-decoration-style: double;\n}}\n\n.db8-block {{\n  font-size: 16pt;\n  font-weight: 700;\n  text-decoration-line: underline;\n  margin-bottom: {}pt;\n}}\n\n.db8-tag {{\n  font-family: \"{}\";\n  font-size: {}pt;\n  font-weight: {};\n  color: {};\n}}\n\n.db8-cite {{\n  font-family: \"{}\";\n  font-size: {}pt;\n  font-weight: {};\n  font-style: {};\n  color: {};\n}}\n\n.db8-normal {{\n  font-size: {}pt;\n  color: {};\n}}\n\n.db8-highlight {{\n  background: {};\n  color: {};\n}}\n\n.db8-emphasis {{\n  font-weight: {};\n  border: 1px solid #000000;\n  padding: 0pt 2pt;\n}}\n\n.db8-underline {{\n  text-decoration-line: underline;\n  text-decoration-color: {};\n  text-decoration-thickness: {}px;\n}}\n\n.db8-shrunk {{\n  font-size: {}%;\n  line-height: {};\n}}\n",
        config.page.width,
        config.page.margin_left,
        config.page.margin_right,
        config.body.font_family,
        config.body.font_size_pt,
        config.body.line_height,
        config.pocket.font_size_pt,
        config.pocket.font_weight,
        config.pocket.spacing_before_pt,
        config.pocket.spacing_after_pt,
        config.hat.font_size_pt,
        config.hat.font_weight,
        config.hat.text_color,
        config.hat.spacing_before_pt,
        config.hat.spacing_after_pt,
        config.block.spacing_after_pt,
        config.tag.font_family,
        config.tag.font_size_pt,
        config.tag.font_weight,
        config.tag.text_color,
        config.body.font_family,
        config.cite.font_size_pt,
        config.tag.font_weight,
        if config.cite.italic {
            "italic"
        } else {
            "normal"
        },
        config.cite.text_color,
        config.normal.font_size_pt,
        config.normal.text_color,
        config.highlight.background,
        config.highlight.text_color,
        config.emphasis.font_weight,
        config.underline.color,
        config.underline.thickness_px,
        config.shrunk.font_size_scale * 100.0,
        config.shrunk.line_height,
    )
}

pub fn style_config_to_typst_theme(config: &Db8StyleConfig) -> String {
    format!(
        "#set page(width: {}, margin: (left: {}, right: {}))\n#set text(font: \"{}\", size: {}pt)\n\n#let db8_pocket(body) = block(spacing: {}pt, stroke: (paint: black, thickness: 1pt), inset: (x: 6pt, y: 2pt))[#align(center)[#text(size: {}pt, weight: {})[#body]]]\n#let db8_hat(body) = block(spacing: {}pt)[#underline(stroke: (paint: rgb(\"{}\"), thickness: 1pt))[#underline(stroke: (paint: rgb(\"{}\"), thickness: 1pt))[#text(size: {}pt, weight: {}, fill: rgb(\"{}\"))[#body]]]]\n#let db8_block(body) = block(spacing: {}pt)[#underline[#text(size: 16pt, weight: 700)[#body]]]\n#let db8_tag(body) = text(font: \"{}\", size: {}pt, weight: {}, fill: rgb(\"{}\"))[#body]\n#let db8_cite(body) = text(font: \"{}\", size: {}pt, weight: {}, style: \"{}\", fill: rgb(\"{}\"))[#body]\n#let db8_normal(body) = text(size: {}pt, fill: rgb(\"{}\"))[#body]\n#let db8_highlight(body) = box(fill: rgb(\"{}\"), inset: (x: 1pt, y: 0pt))[#text(fill: rgb(\"{}\"))[#body]]\n#let db8_emphasis(body) = box(stroke: (paint: black, thickness: 1pt), inset: (x: 2pt, y: 0pt))[#text(weight: {})[#body]]\n#let db8_underline(body) = underline(stroke: (paint: rgb(\"{}\"), thickness: {}pt))[#body]\n#let db8_shrunk(body) = text(size: {}pt)[#body]\n",
        config.page.width,
        config.page.margin_left,
        config.page.margin_right,
        config.body.font_family,
        config.body.font_size_pt,
        config.pocket.spacing_after_pt,
        config.pocket.font_size_pt,
        config.pocket.font_weight,
        config.hat.spacing_after_pt,
        config.hat.text_color,
        config.hat.text_color,
        config.hat.font_size_pt,
        config.hat.font_weight,
        config.hat.text_color,
        config.block.spacing_after_pt,
        config.tag.font_family,
        config.tag.font_size_pt,
        config.tag.font_weight,
        config.tag.text_color,
        config.body.font_family,
        config.cite.font_size_pt,
        config.tag.font_weight,
        if config.cite.italic {
            "italic"
        } else {
            "normal"
        },
        config.cite.text_color,
        config.normal.font_size_pt,
        config.normal.text_color,
        config.highlight.background,
        config.highlight.text_color,
        config.emphasis.font_weight,
        config.underline.color,
        config.underline.thickness_px,
        config.body.font_size_pt * config.shrunk.font_size_scale,
    )
}

pub fn db8_document_to_typst(document: &Db8Document, config: &Db8StyleConfig) -> String {
    let mut out = String::new();
    out.push_str(&style_config_to_typst_theme(config));
    out.push('\n');

    for block in &document.blocks {
        if block.spans.is_empty() {
            out.push_str("#linebreak()\n");
            continue;
        }

        let block_text = block
            .spans
            .iter()
            .map(span_to_typst)
            .collect::<Vec<_>>()
            .join("");

        out.push_str(&format!("{}\n", block_text));
    }

    out
}

fn span_to_typst(span: &Db8Span) -> String {
    let mut content = escape_typst(&span.text);
    content = wrap_typst_style(span.styles, content);
    content
}

fn wrap_typst_style(styles: StyleSet, content: String) -> String {
    let mut wrapped = content;

    if styles.shrunk {
        wrapped = format!("#db8_shrunk([{}])", wrapped);
    }
    if styles.underline {
        wrapped = format!("#db8_underline([{}])", wrapped);
    }
    if styles.emphasis {
        wrapped = format!("#db8_emphasis([{}])", wrapped);
    }
    if styles.highlight {
        wrapped = format!("#db8_highlight([{}])", wrapped);
    }
    if styles.cite {
        wrapped = format!("#db8_cite([{}])", wrapped);
    }
    if styles.tag {
        wrapped = format!("#db8_tag([{}])", wrapped);
    }
    if styles.block {
        wrapped = format!("#db8_block([{}])", wrapped);
    }
    if styles.hat {
        wrapped = format!("#db8_hat([{}])", wrapped);
    }
    if styles.pocket {
        wrapped = format!("#db8_pocket([{}])", wrapped);
    }

    if styles.is_plain() {
        format!("#db8_normal([{}])", wrapped)
    } else {
        wrapped
    }
}

fn escape_typst(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}
