use super::*;

impl DebateEditor {
    pub(super) fn render_document_pane(
        &self,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .child(self.render_document_toolbar(colors, cx))
            .when_some(self.export_status.as_ref(), |this, status| {
                this.child(
                    div()
                        .px_4()
                        .py_2()
                        .text_size(px(11.0))
                        .text_color(colors.text_muted)
                        .border_b_1()
                        .border_color(colors.border)
                        .child(status.clone()),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .child(
                        div()
                            .id("document-scroll")
                            .flex_1()
                            .min_w(px(0.0))
                            .overflow_scroll()
                            .bg(colors.background)
                            .when_some(self.opened_document.as_ref(), |this, document| {
                                match self.document_view_mode {
                                    DocumentViewMode::Raw => {
                                        this.child(self.render_raw_document(document, colors, cx))
                                    }
                                    DocumentViewMode::Render => this.child(
                                        self.render_editable_db8_document(document, colors, cx),
                                    ),
                                }
                            })
                            .when(self.opened_document.is_none(), |this| {
                                this.flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(18.0))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("No document open"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .text_color(colors.text_muted)
                                            .child("Open a .db8 file from the project tree."),
                                    )
                            }),
                    )
                    .when(
                        self.document_view_mode == DocumentViewMode::Render
                            && self.opened_document.is_some(),
                        |this| {
                            this.child(self.render_document_outline(
                                self.opened_document.as_ref().unwrap(),
                                colors,
                                cx,
                            ))
                        },
                    ),
            )
    }

    fn render_document_toolbar(
        &self,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let title = self
            .opened_document
            .as_ref()
            .and_then(|document| document.path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Editor")
            .to_string();

        div()
            .h(px(40.0))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .border_b_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .px_4()
            .child(
                div()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .text_overflow(gpui::TextOverflow::Truncate("".into()))
                    .text_size(px(13.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title),
            )
            .child(div().flex_none().flex().items_center().gap_1().when(
                self.document_view_mode == DocumentViewMode::Render,
                |this| {
                    this.child(
                        div()
                            .id("render-style-normal")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(if self.cursor_has_style("normal") {
                                colors.accent
                            } else {
                                colors.surface_elevated
                            })
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_normal_style))
                            .child("Normal"),
                    )
                    .child(
                        div()
                            .id("render-style-pocket")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(if self.cursor_has_style("pocket") {
                                colors.accent
                            } else {
                                colors.surface_elevated
                            })
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_pocket_style))
                            .child("Pocket"),
                    )
                    .child(
                        div()
                            .id("render-style-hat")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(if self.cursor_has_style("hat") {
                                colors.accent
                            } else {
                                colors.surface_elevated
                            })
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_hat_style))
                            .child("Hat"),
                    )
                    .child(
                        div()
                            .id("render-style-block")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(if self.cursor_has_style("block") {
                                colors.accent
                            } else {
                                colors.surface_elevated
                            })
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_block_style))
                            .child("Block"),
                    )
                    .child(
                        div()
                            .id("render-style-tag")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(if self.cursor_has_style("tag") {
                                colors.accent
                            } else {
                                colors.surface_elevated
                            })
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_tag_style))
                            .child("Tag"),
                    )
                    .child(
                        div()
                            .id("render-style-cite")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(if self.cursor_has_style("cite") {
                                colors.accent
                            } else {
                                colors.surface_elevated
                            })
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_cite_style))
                            .child("Cite"),
                    )
                    .child(
                        div()
                            .id("render-style-highlight")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(
                                if self.cursor_has_style("highlight")
                                    || self.render_active_styles.highlight
                                {
                                    colors.accent
                                } else {
                                    colors.surface_elevated
                                },
                            )
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(Self::apply_highlight_style),
                            )
                            .child("H"),
                    )
                    .child(
                        div()
                            .id("render-style-emphasis")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(
                                if self.cursor_has_style("emphasis")
                                    || self.render_active_styles.emphasis
                                {
                                    colors.accent
                                } else {
                                    colors.surface_elevated
                                },
                            )
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(Self::apply_emphasis_style),
                            )
                            .child("B"),
                    )
                    .child(
                        div()
                            .id("render-style-underline")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(
                                if self.cursor_has_style("underline")
                                    || self.render_active_styles.underline
                                {
                                    colors.accent
                                } else {
                                    colors.surface_elevated
                                },
                            )
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(Self::apply_underline_style),
                            )
                            .child("U"),
                    )
                    .child(
                        div()
                            .id("render-style-shrunk")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(
                                if self.cursor_has_style("shrunk")
                                    || self.render_active_styles.shrunk
                                {
                                    colors.accent
                                } else {
                                    colors.surface_elevated
                                },
                            )
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::apply_shrunk_style))
                            .child("S"),
                    )
                    .child(
                        div()
                            .id("render-export-typ")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.surface_elevated)
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(Self::export_typst_source),
                            )
                            .child(".typ"),
                    )
                    .child(
                        div()
                            .id("render-export-pdf")
                            .h(px(26.0))
                            .px_2()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.surface_elevated)
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::export_pdf))
                            .child("PDF"),
                    )
                },
            ))
    }

    fn render_raw_document(
        &self,
        document: &OpenedDocument,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("raw-editor")
            .size_full()
            .pt_4()
            .pb_4()
            .flex()
            .flex_col()
            .track_focus(&self.raw_editor_focus)
            .font_family("DejaVu Sans Mono")
            .text_size(px(13.0))
            .text_color(colors.text)
            .cursor_pointer()
            .children(
                split_raw_lines(&document.raw)
                    .into_iter()
                    .enumerate()
                    .map(|(index, line)| self.render_raw_line(index, line, colors, cx)),
            )
            .child(
                div()
                    .id("raw-editor-filler")
                    .flex_1()
                    .min_h(px(80.0))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(Self::start_raw_document_mouse_selection),
                    )
                    .on_mouse_move(cx.listener(Self::update_raw_document_mouse_selection))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(Self::finish_raw_mouse_selection),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(Self::finish_raw_mouse_selection),
                    ),
            )
    }

    fn render_raw_line(
        &self,
        index: usize,
        line: String,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_cursor_line = self.raw_cursor.line == index;
        let line_start = self
            .opened_document
            .as_ref()
            .map_or(0, |document| raw_line_start_offset(&document.raw, index));
        let line_end = line_start + line.len();
        let selected_range = self
            .raw_selection
            .filter(|selection| !selection.is_empty())
            .map(RawSelection::ordered)
            .and_then(|(start, end)| {
                let selected_start = start.max(line_start);
                let selected_end = end.min(line_end);

                if selected_start < selected_end {
                    Some((selected_start - line_start, selected_end - line_start))
                } else {
                    None
                }
            });
        let column = clamp_to_char_boundary(&line, self.raw_cursor.column);

        div()
            .id(("raw-line", index))
            .w_full()
            .px_4()
            .min_h(px(22.0))
            .flex()
            .items_center()
            .whitespace_nowrap()
            .when(is_cursor_line, |this| this.bg(colors.surface_elevated))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event, window, cx| {
                    this.start_raw_mouse_selection(index, event, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(move |this, event, window, cx| {
                this.update_raw_mouse_selection(index, event, window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(Self::finish_raw_mouse_selection),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(Self::finish_raw_mouse_selection),
            )
            .child(
                div()
                    .w(px(48.0))
                    .flex_none()
                    .pr_3()
                    .text_size(px(12.0))
                    .text_color(colors.text_muted)
                    .child((index + 1).to_string()),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .min_w(px(0.0))
                    .children(render_raw_line_segments(
                        &line,
                        column,
                        is_cursor_line && selected_range.is_none(),
                        selected_range,
                        colors,
                    )),
            )
            .into_any_element()
    }

    fn render_editable_db8_document(
        &self,
        document: &OpenedDocument,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("db8-render-editor")
            .size_full()
            .min_h_full()
            .p_4()
            .flex()
            .flex_col()
            .track_focus(&self.render_editor_focus)
            .bg(colors.background)
            .text_color(colors.text)
            .font_family("Calibri")
            .text_size(px(11.0 * 1.33))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(Self::start_render_end_mouse_selection),
            )
            .on_mouse_move(cx.listener(Self::update_render_end_mouse_selection))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(Self::finish_render_mouse_selection),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(Self::finish_render_mouse_selection),
            )
            .children(
                document
                    .parsed
                    .blocks
                    .iter()
                    .enumerate()
                    .map(|(index, block)| self.render_editable_db8_block(index, block, colors, cx)),
            )
    }

    fn render_editable_db8_block(
        &self,
        line_index: usize,
        block: &Db8Block,
        colors: AppColors,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_cursor_line = self.render_cursor.line == line_index;
        let block_class = semantic_block_class(block);
        let line_height_px = 11.0 * 1.33 * self.style_config.body.line_height;
        let spacing_before = match block_class {
            "pocket" => self.style_config.pocket.spacing_before_pt * 1.33,
            "hat" => self.style_config.hat.spacing_before_pt * 1.33,
            _ => 0.0,
        };
        let spacing_after = match block_class {
            "pocket" => self.style_config.pocket.spacing_after_pt * 1.33,
            "hat" => self.style_config.hat.spacing_after_pt * 1.33,
            "block" => self.style_config.block.spacing_after_pt * 1.33,
            _ => 0.0,
        };
        let line_start = self.opened_document.as_ref().map_or(0, |document| {
            rendered_line_start_offset(&document.parsed, line_index)
        });
        let line_text = rendered_line_text(block);
        let line_end = line_start + line_text.len();
        let selected_range = self
            .render_selection
            .filter(|selection| !selection.is_empty())
            .map(RawSelection::ordered)
            .and_then(|(start, end)| {
                let selected_start = start.max(line_start);
                let selected_end = end.min(line_end);
                if selected_start < selected_end {
                    Some((selected_start - line_start, selected_end - line_start))
                } else {
                    None
                }
            });

        div()
            .id(("db8-render-line", line_index))
            .w_full()
            .min_h(px(line_height_px.max(18.0)))
            .mt(px(spacing_before))
            .mb(px(spacing_after))
            .flex()
            .items_center()
            .gap_1()
            .text_color(gpui::black())
            .when(semantic_block_class(block) == "pocket", |this| {
                this.justify_center()
                    .border_1()
                    .border_color(gpui::black())
                    .px_2()
                    .py_1()
            })
            .when(semantic_block_class(block) == "hat", |this| {
                this.justify_center()
            })
            .when(semantic_block_class(block) == "block", |this| {
                this.justify_center()
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event, window, cx| {
                    this.start_render_mouse_selection(line_index, event, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(move |this, event, window, cx| {
                this.update_render_mouse_selection(line_index, event, window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(Self::finish_render_mouse_selection),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(Self::finish_render_mouse_selection),
            )
            .child(
                div()
                    .id(SharedString::from(format!(
                        "db8-line-class-{}",
                        semantic_block_class(block)
                    )))
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_0()
                    .children(render_db8_line_segments(
                        block,
                        self.render_cursor.column,
                        is_cursor_line && selected_range.is_none(),
                        selected_range,
                        &self.style_config,
                        colors,
                    )),
            )
    }
    pub(super) fn export_typst_source(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let typ_path = document.path.with_extension("typ");
        let typ_source = db8_document_to_typst(&document.parsed, &self.style_config);
        match std::fs::write(&typ_path, typ_source) {
            Ok(_) => self.export_status = Some(format!("Generated {}", typ_path.display())),
            Err(error) => self.export_status = Some(format!("Export failed: {}", error)),
        }
        cx.notify();
    }

    pub(super) fn export_pdf(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let typ_path = document.path.with_extension("typ");
        let pdf_path = document.path.with_extension("pdf");
        let typ_source = db8_document_to_typst(&document.parsed, &self.style_config);
        if std::fs::write(&typ_path, typ_source).is_err() {
            self.export_status = Some("Failed to write .typ file".to_string());
            cx.notify();
            return;
        }

        let typst_installed = Command::new("typst").arg("--version").output().is_ok();
        if !typst_installed {
            self.export_status = Some(format!(
                "Generated {}. Install Typst to enable PDF export.",
                typ_path.display()
            ));
            cx.notify();
            return;
        }

        let status = Command::new("typst")
            .arg("compile")
            .arg(&typ_path)
            .arg(&pdf_path)
            .status();

        self.export_status = Some(match status {
            Ok(result) if result.success() => format!("Generated {}", pdf_path.display()),
            Ok(_) => format!(
                "Typst compile failed. .typ file is available at {}",
                typ_path.display()
            ),
            Err(error) => format!("Failed to execute Typst: {}", error),
        });
        cx.notify();
    }
}
