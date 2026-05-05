use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Db8Document {
    #[serde(default = "default_version")]
    pub version: u32,
    pub blocks: Vec<Db8Block>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Db8Block {
    #[serde(default)]
    pub source_line: usize,
    #[serde(default)]
    pub style: BlockStyle,
    pub spans: Vec<Db8Span>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Db8Span {
    pub text: String,
    #[serde(default)]
    pub styles: StyleSet,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockStyle {
    #[default]
    Normal,
    Pocket,
    Hat,
    Block,
    Tag,
    Cite,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleSet {
    #[serde(default, skip_serializing_if = "is_false")]
    pub pocket: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hat: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub block: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub tag: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub cite: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub emphasis: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub shrunk: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub highlight: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn default_version() -> u32 {
    1
}

impl StyleSet {
    pub fn is_plain(self) -> bool {
        !self.pocket
            && !self.hat
            && !self.block
            && !self.tag
            && !self.cite
            && !self.emphasis
            && !self.underline
            && !self.shrunk
            && !self.highlight
    }
}

impl Db8Document {
    pub fn parse(raw: &str) -> Self {
        if raw.trim().is_empty() {
            return Self::empty();
        }

        serde_json::from_str::<Db8Document>(raw).unwrap_or_else(|_| Self::empty())
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::empty();
        }

        rmp_serde::from_slice::<Db8Document>(bytes)
            .or_else(|_| serde_json::from_slice::<Db8Document>(bytes))
            .unwrap_or_else(|_| Self::empty())
    }

    pub fn empty() -> Self {
        Self {
            version: 1,
            blocks: vec![Db8Block {
                source_line: 0,
                style: BlockStyle::Normal,
                spans: vec![Db8Span {
                    text: String::new(),
                    styles: StyleSet::default(),
                }],
            }],
        }
    }

    pub fn to_db8(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|_| "{\"version\":1,\"blocks\":[]}".to_string())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec_named(self).unwrap_or_else(|_| self.to_db8().into_bytes())
    }
}
