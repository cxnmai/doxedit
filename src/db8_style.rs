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
                font_weight: 1200,
                spacing_before_pt: 12.0,
                spacing_after_pt: 8.0,
            },
            hat: HatStyle {
                font_size_pt: 22.0,
                font_weight: 1200,
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
                font_weight: 1200,
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
            emphasis: EmphasisStyle { font_weight: 1200 },
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
        ".db8-editor {{\n  width: {};\n  margin-left: {};\n  margin-right: {};\n  font-family: \"{}\";\n  font-size: {}pt;\n  line-height: {};\n}}\n\n.db8-pocket {{\n  font-size: {}pt;\n  font-weight: {};\n  margin-top: {}pt;\n  margin-bottom: {}pt;\n  border: 1px solid #000000;\n  padding: 2pt 6pt;\n  text-align: center;\n}}\n\n.db8-hat {{\n  font-size: {}pt;\n  font-weight: {};\n  color: {};\n  margin-top: {}pt;\n  margin-bottom: {}pt;\n  text-decoration-line: underline;\n  text-decoration-style: double;\n}}\n\n.db8-block {{\n  font-size: 16pt;\n  font-weight: 1200;\n  text-decoration-line: underline;\n  margin-bottom: {}pt;\n}}\n\n.db8-tag {{\n  font-family: \"{}\";\n  font-size: {}pt;\n  font-weight: {};\n  color: {};\n}}\n\n.db8-cite {{\n  font-family: \"{}\";\n  font-size: {}pt;\n  font-weight: {};\n  font-style: {};\n  color: {};\n}}\n\n.db8-normal {{\n  font-size: {}pt;\n  color: {};\n}}\n\n.db8-highlight {{\n  background: {};\n  color: {};\n}}\n\n.db8-emphasis {{\n  font-weight: {};\n  border: 1px solid #000000;\n  padding: 0pt 2pt;\n}}\n\n.db8-underline {{\n  text-decoration-line: underline;\n  text-decoration-color: {};\n  text-decoration-thickness: {}px;\n}}\n\n.db8-shrunk {{\n  font-size: {}%;\n  line-height: {};\n}}\n",
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
        config.tag.font_family,
        config.tag.font_size_pt,
        config.tag.font_weight,
        "normal",
        config.tag.text_color,
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
