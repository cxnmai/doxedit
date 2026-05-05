use super::*;

struct OutlinePocket {
    index: usize,
    title: String,
    hats: Vec<OutlineHat>,
    blocks: Vec<OutlineBlock>,
    tags: Vec<OutlineTag>,
}

struct OutlineHat {
    index: usize,
    title: String,
    blocks: Vec<OutlineBlock>,
    tags: Vec<OutlineTag>,
}

struct OutlineBlock {
    index: usize,
    title: String,
    tags: Vec<OutlineTag>,
}

struct OutlineTag {
    index: usize,
    title: String,
}

impl DebateEditor {
    pub(super) fn render_document_outline(
        &self,
        document: &OpenedDocument,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let outline = build_document_outline(&document.parsed);

        div()
            .id("document-outline")
            .w(px(260.0))
            .h_full()
            .flex_none()
            .border_l_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(36.0))
                    .flex()
                    .items_center()
                    .px_3()
                    .border_b_1()
                    .border_color(colors.border)
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("Document Tree"),
            )
            .child(
                div()
                    .id("document-outline-scroll")
                    .flex_1()
                    .overflow_scroll()
                    .py_2()
                    .when(outline.is_empty(), |this| {
                        this.child(
                            div()
                                .px_3()
                                .py_2()
                                .text_size(px(12.0))
                                .text_color(colors.text_muted)
                                .child("No pocket/hat/block/tag structure."),
                        )
                    })
                    .children(
                        outline
                            .iter()
                            .map(|pocket| self.render_outline_pocket(pocket, colors, cx)),
                    ),
            )
    }

    fn render_outline_pocket(
        &self,
        pocket: &OutlinePocket,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = outline_key("pocket", pocket.index);
        let expanded = self.expanded_document_outline.contains(&key);
        div()
            .flex()
            .flex_col()
            .child(self.render_outline_row(
                key,
                expanded,
                !pocket.hats.is_empty() || !pocket.blocks.is_empty() || !pocket.tags.is_empty(),
                pocket.index,
                "Pocket",
                &pocket.title,
                0,
                colors,
                cx,
            ))
            .when(expanded, |this| {
                this.children(
                    pocket
                        .hats
                        .iter()
                        .map(|hat| self.render_outline_hat(hat, colors, cx)),
                )
                .children(
                    pocket
                        .blocks
                        .iter()
                        .map(|block| self.render_outline_block(block, colors, cx)),
                )
                .children(
                    pocket
                        .tags
                        .iter()
                        .map(|tag| self.render_outline_tag(tag, 1, colors, cx)),
                )
            })
            .into_any_element()
    }

    fn render_outline_hat(
        &self,
        hat: &OutlineHat,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = outline_key("hat", hat.index);
        let expanded = self.expanded_document_outline.contains(&key);
        div()
            .flex()
            .flex_col()
            .child(self.render_outline_row(
                key,
                expanded,
                !hat.blocks.is_empty() || !hat.tags.is_empty(),
                hat.index,
                "Hat",
                &hat.title,
                1,
                colors,
                cx,
            ))
            .when(expanded, |this| {
                this.children(
                    hat.blocks
                        .iter()
                        .map(|block| self.render_outline_block(block, colors, cx)),
                )
                .children(
                    hat.tags
                        .iter()
                        .map(|tag| self.render_outline_tag(tag, 2, colors, cx)),
                )
            })
            .into_any_element()
    }

    fn render_outline_block(
        &self,
        block: &OutlineBlock,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = outline_key("block", block.index);
        let expanded = self.expanded_document_outline.contains(&key);
        div()
            .flex()
            .flex_col()
            .child(self.render_outline_row(
                key,
                expanded,
                !block.tags.is_empty(),
                block.index,
                "Block",
                &block.title,
                2,
                colors,
                cx,
            ))
            .when(expanded, |this| {
                this.children(
                    block
                        .tags
                        .iter()
                        .map(|tag| self.render_outline_tag(tag, 3, colors, cx)),
                )
            })
            .into_any_element()
    }

    fn render_outline_tag(
        &self,
        tag: &OutlineTag,
        depth: usize,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_outline_row(
            outline_key("tag", tag.index),
            false,
            false,
            tag.index,
            "Tag",
            &tag.title,
            depth,
            colors,
            cx,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_outline_row(
        &self,
        key: String,
        expanded: bool,
        expandable: bool,
        line_index: usize,
        _label: &'static str,
        title: &str,
        depth: usize,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.render_cursor.line == line_index;
        let left_padding = 10.0 + depth as f32 * 16.0;
        div()
            .id(SharedString::from(format!("outline-row-{}", key)))
            .min_h(px(26.0))
            .flex()
            .items_center()
            .gap_1()
            .pl(px(left_padding))
            .pr_2()
            .text_size(px(12.0))
            .bg(if selected { colors.surface_elevated } else { colors.surface })
            .hover(move |style| style.bg(colors.surface_elevated))
            .child(
                div()
                    .w(px(12.0))
                    .flex_none()
                    .text_color(colors.text_muted)
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.toggle_outline_row(key.clone(), expandable, cx);
                        }),
                    )
                    .child(if expandable { if expanded { "▾" } else { "▸" } } else { "" }),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .text_overflow(gpui::TextOverflow::Truncate("".into()))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, window, cx| {
                            this.activate_outline_row(line_index, window, cx);
                        }),
                    )
                    .child(title.to_string()),
            )
            .into_any_element()
    }

    fn toggle_outline_row(&mut self, key: String, expandable: bool, cx: &mut Context<Self>) {
        if expandable && !self.expanded_document_outline.remove(&key) {
            self.expanded_document_outline.insert(key);
        }
        cx.notify();
    }

    fn activate_outline_row(
        &mut self,
        line_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.render_cursor = RawCursor {
            line: line_index,
            column: 0,
        };
        self.render_selection = None;
        window.focus(&self.render_editor_focus);
        cx.notify();
    }
}

fn build_document_outline(document: &Db8Document) -> Vec<OutlinePocket> {
    let mut pockets = Vec::new();
    let mut current_pocket: Option<OutlinePocket> = None;
    let mut current_hat: Option<OutlineHat> = None;
    let mut current_block: Option<OutlineBlock> = None;

    for (index, block) in document.blocks.iter().enumerate() {
        match block.style {
            BlockStyle::Pocket => {
                flush_block(&mut current_pocket, &mut current_hat, &mut current_block);
                flush_hat(&mut current_pocket, &mut current_hat);
                flush_pocket(&mut pockets, &mut current_pocket);
                current_pocket = Some(OutlinePocket {
                    index,
                    title: outline_title(block),
                    hats: Vec::new(),
                    blocks: Vec::new(),
                    tags: Vec::new(),
                });
            }
            BlockStyle::Hat => {
                ensure_pocket(&mut current_pocket, index);
                flush_block(&mut current_pocket, &mut current_hat, &mut current_block);
                flush_hat(&mut current_pocket, &mut current_hat);
                current_hat = Some(OutlineHat {
                    index,
                    title: outline_title(block),
                    blocks: Vec::new(),
                    tags: Vec::new(),
                });
            }
            BlockStyle::Block => {
                ensure_pocket(&mut current_pocket, index);
                flush_block(&mut current_pocket, &mut current_hat, &mut current_block);
                current_block = Some(OutlineBlock {
                    index,
                    title: outline_title(block),
                    tags: Vec::new(),
                });
            }
            BlockStyle::Tag => {
                ensure_pocket(&mut current_pocket, index);
                let tag = OutlineTag {
                    index,
                    title: outline_title(block),
                };
                if let Some(block_outline) = current_block.as_mut() {
                    block_outline.tags.push(tag);
                } else if let Some(hat) = current_hat.as_mut() {
                    hat.tags.push(tag);
                } else if let Some(pocket) = current_pocket.as_mut() {
                    pocket.tags.push(tag);
                }
            }
            BlockStyle::Normal | BlockStyle::Cite => {}
        }
    }

    flush_block(&mut current_pocket, &mut current_hat, &mut current_block);
    flush_hat(&mut current_pocket, &mut current_hat);
    flush_pocket(&mut pockets, &mut current_pocket);
    pockets
}

fn flush_pocket(pockets: &mut Vec<OutlinePocket>, current_pocket: &mut Option<OutlinePocket>) {
    if let Some(pocket) = current_pocket.take() {
        pockets.push(pocket);
    }
}

fn flush_hat(current_pocket: &mut Option<OutlinePocket>, current_hat: &mut Option<OutlineHat>) {
    if let Some(hat) = current_hat.take()
        && let Some(pocket) = current_pocket.as_mut()
    {
        pocket.hats.push(hat);
    }
}

fn flush_block(
    current_pocket: &mut Option<OutlinePocket>,
    current_hat: &mut Option<OutlineHat>,
    current_block: &mut Option<OutlineBlock>,
) {
    if let Some(block) = current_block.take() {
        if let Some(hat) = current_hat.as_mut() {
            hat.blocks.push(block);
        } else if let Some(pocket) = current_pocket.as_mut() {
            pocket.blocks.push(block);
        }
    }
}

fn ensure_pocket(current_pocket: &mut Option<OutlinePocket>, index: usize) {
    if current_pocket.is_none() {
        *current_pocket = Some(OutlinePocket {
            index,
            title: "Untitled pocket".to_string(),
            hats: Vec::new(),
            blocks: Vec::new(),
            tags: Vec::new(),
        });
    }
}

fn outline_title(block: &Db8Block) -> String {
    let text = rendered_line_text(block).trim().to_string();
    if text.is_empty() {
        "Untitled".to_string()
    } else if text.chars().count() > 48 {
        format!("{}…", text.chars().take(48).collect::<String>())
    } else {
        text
    }
}

pub(super) fn default_document_outline_expansion(document: &Db8Document) -> HashSet<String> {
    let mut expanded = HashSet::new();
    for (index, block) in document.blocks.iter().enumerate() {
        match block.style {
            BlockStyle::Pocket => {
                expanded.insert(outline_key("pocket", index));
            }
            BlockStyle::Hat => {
                expanded.insert(outline_key("hat", index));
            }
            BlockStyle::Block => {
                expanded.insert(outline_key("block", index));
            }
            BlockStyle::Normal | BlockStyle::Tag | BlockStyle::Cite => {}
        }
    }
    expanded
}

fn outline_key(kind: &str, index: usize) -> String {
    format!("{}:{}", kind, index)
}
