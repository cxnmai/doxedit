#[derive(Clone)]
pub struct Db8Document {
    pub blocks: Vec<Db8Block>,
}

#[derive(Clone)]
pub struct Db8Block {
    pub source_line: usize,
    pub spans: Vec<Db8Span>,
}

#[derive(Clone)]
pub struct Db8Span {
    pub text: String,
    pub styles: StyleSet,
}

#[derive(Clone, Copy, Default)]
pub struct StyleSet {
    pub pocket: bool,
    pub hat: bool,
    pub block: bool,
    pub tag: bool,
    pub cite: bool,
    pub emphasis: bool,
    pub underline: bool,
    pub shrunk: bool,
    pub highlight: bool,
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

    pub fn tokens(self) -> Vec<&'static str> {
        let mut tokens = Vec::new();
        if self.pocket {
            tokens.push("pocket");
        }
        if self.hat {
            tokens.push("hat");
        }
        if self.block {
            tokens.push("block");
        }
        if self.tag {
            tokens.push("tag");
        }
        if self.cite {
            tokens.push("cite");
        }
        if self.emphasis {
            tokens.push("emphasis");
        }
        if self.underline {
            tokens.push("underline");
        }
        if self.shrunk {
            tokens.push("shrunk");
        }
        if self.highlight {
            tokens.push("highlight");
        }
        tokens
    }
}

impl Db8Document {
    pub fn parse(raw: &str) -> Self {
        Self {
            blocks: raw
                .lines()
                .enumerate()
                .map(|(line_index, line)| Db8Block {
                    source_line: line_index,
                    spans: parse_spans(line),
                })
                .collect(),
        }
    }

    pub fn to_db8(&self) -> String {
        self.blocks
            .iter()
            .map(|block| serialize_spans(&block.spans))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn serialize_spans(spans: &[Db8Span]) -> String {
    spans
        .iter()
        .map(|span| {
            if span.styles.is_plain() {
                span.text.clone()
            } else {
                let style_tokens = span.styles.tokens().join(" ");
                format!("[{}: {}]", style_tokens, span.text)
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn parse_spans(line: &str) -> Vec<Db8Span> {
    let mut spans = Vec::new();
    let mut rest = line;

    while let Some(open_index) = rest.find('[') {
        let before = &rest[..open_index];
        push_normal_span(&mut spans, before);

        let after_open = &rest[open_index + 1..];
        let Some(close_index) = after_open.find(']') else {
            push_normal_span(&mut spans, &rest[open_index..]);
            return spans;
        };

        let candidate = &after_open[..close_index];
        let Some((style_part, text_part)) = candidate.split_once(':') else {
            push_normal_span(&mut spans, &rest[open_index..open_index + close_index + 2]);
            rest = &after_open[close_index + 1..];
            continue;
        };

        let styles = parse_styles(style_part);
        spans.push(Db8Span {
            text: text_part.trim_start().to_string(),
            styles,
        });
        rest = &after_open[close_index + 1..];
    }

    push_normal_span(&mut spans, rest);

    if spans.is_empty() {
        spans.push(Db8Span {
            text: String::new(),
            styles: StyleSet::default(),
        });
    }

    spans
}

fn push_normal_span(spans: &mut Vec<Db8Span>, text: &str) {
    if !text.is_empty() {
        spans.push(Db8Span {
            text: text.to_string(),
            styles: StyleSet::default(),
        });
    }
}

fn parse_styles(style_part: &str) -> StyleSet {
    let mut styles = StyleSet::default();

    for token in style_part.split_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "pocket" => styles.pocket = true,
            "hat" => styles.hat = true,
            "block" => styles.block = true,
            "tag" => styles.tag = true,
            "cite" => styles.cite = true,
            "bold" | "emphasis" => styles.emphasis = true,
            "underline" | "underlined" => styles.underline = true,
            "small" | "shrunk" => styles.shrunk = true,
            "highlight" | "highlighted" => styles.highlight = true,
            _ => {}
        }
    }

    styles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_composable_inline_styles() {
        let document = Db8Document::parse(
            "Normal [emphasis underline cite: Smith 24] and [highlight shrunk: key text]",
        );

        let block = &document.blocks[0];
        assert_eq!(block.spans.len(), 4);
        assert_eq!(block.spans[1].text, "Smith 24");
        assert!(block.spans[1].styles.emphasis);
        assert!(block.spans[1].styles.underline);
        assert!(block.spans[1].styles.cite);
        assert_eq!(block.spans[3].text, "key text");
        assert!(block.spans[3].styles.highlight);
        assert!(block.spans[3].styles.shrunk);
    }

    #[test]
    fn supports_legacy_alias_tokens() {
        let document = Db8Document::parse("[bold small: Alias handling]");
        let span = &document.blocks[0].spans[0];
        assert!(span.styles.emphasis);
        assert!(span.styles.shrunk);
    }

    #[test]
    fn serializes_back_to_db8_line_syntax() {
        let document = Db8Document::parse("A [tag emphasis: claim] line");
        assert_eq!(document.to_db8(), "A [tag emphasis: claim] line");
    }
}
