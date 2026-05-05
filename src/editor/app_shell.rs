use super::*;

impl Render for DebateEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.color_mode.colors();

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(colors.background)
            .text_color(colors.text)
            .child(
                div()
                    .h(px(40.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(colors.surface)
                    .border_b_1()
                    .border_color(colors.border)
                    .px_4()
                    .text_size(px(15.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("project-tree-toggle")
                                    .size(px(30.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_sm()
                                    .bg(colors.surface_elevated)
                                    .border_1()
                                    .border_color(colors.border)
                                    .text_size(px(14.0))
                                    .text_color(colors.text)
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_click(cx.listener(Self::toggle_sidebar))
                                    .child(if self.sidebar_open { "◧" } else { "◨" }),
                            )
                            .child("Doxedit"),
                    )
                    .child(
                        div()
                            .id("settings-button")
                            .size(px(30.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .bg(colors.surface_elevated)
                            .border_1()
                            .border_color(colors.border)
                            .text_size(px(16.0))
                            .text_color(colors.text)
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.background))
                            .on_click(cx.listener(Self::toggle_settings))
                            .child("⚙"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .min_h(px(0.0))
                    .when(self.sidebar_open, |this| {
                        this.child(self.render_project_tree(colors, cx))
                    })
                    .child(self.render_document_pane(colors, cx)),
            )
            .when(self.settings_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(px(52.0))
                        .right(px(16.0))
                        .w(px(260.0))
                        .flex()
                        .flex_col()
                        .gap_3()
                        .rounded_sm()
                        .border_1()
                        .border_color(colors.border)
                        .bg(colors.surface)
                        .p_3()
                        .shadow_lg()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child("Settings"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap_3()
                                        .child(div().text_size(px(13.0)).child("Appearance"))
                                        .child(
                                            div()
                                                .id("theme-toggle")
                                                .flex_none()
                                                .w(px(72.0))
                                                .h(px(28.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .rounded_sm()
                                                .border_1()
                                                .border_color(colors.border)
                                                .bg(colors.surface_elevated)
                                                .text_size(px(12.0))
                                                .text_color(colors.text)
                                                .cursor_pointer()
                                                .hover(|style| style.bg(colors.background))
                                                .on_click(cx.listener(Self::toggle_color_mode))
                                                .child(self.color_mode.label()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(colors.text_muted)
                                        .child("Switch between light and dark mode."),
                                )
                                .child(
                                    div()
                                        .mt_2()
                                        .text_size(px(13.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child("DB8 style config"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .text_size(px(12.0))
                                        .child("Body font size")
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .h(px(24.0))
                                                        .px_2()
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(colors.border)
                                                        .bg(colors.surface_elevated)
                                                        .cursor_pointer()
                                                        .id("body-font-minus")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, e, w, cx| {
                                                                this.adjust_body_font_size(
                                                                    -0.5, e, w, cx,
                                                                )
                                                            }),
                                                        )
                                                        .child("-"),
                                                )
                                                .child(div().text_size(px(12.0)).child(format!(
                                                    "{:.1}pt",
                                                    self.style_config.body.font_size_pt
                                                )))
                                                .child(
                                                    div()
                                                        .h(px(24.0))
                                                        .px_2()
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(colors.border)
                                                        .bg(colors.surface_elevated)
                                                        .cursor_pointer()
                                                        .id("body-font-plus")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, e, w, cx| {
                                                                this.adjust_body_font_size(
                                                                    0.5, e, w, cx,
                                                                )
                                                            }),
                                                        )
                                                        .child("+"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .text_size(px(12.0))
                                        .child("Line height")
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .h(px(24.0))
                                                        .px_2()
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(colors.border)
                                                        .bg(colors.surface_elevated)
                                                        .cursor_pointer()
                                                        .id("line-height-minus")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, e, w, cx| {
                                                                this.adjust_body_line_height(
                                                                    -0.05, e, w, cx,
                                                                )
                                                            }),
                                                        )
                                                        .child("-"),
                                                )
                                                .child(div().text_size(px(12.0)).child(format!(
                                                    "{:.2}",
                                                    self.style_config.body.line_height
                                                )))
                                                .child(
                                                    div()
                                                        .h(px(24.0))
                                                        .px_2()
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(colors.border)
                                                        .bg(colors.surface_elevated)
                                                        .cursor_pointer()
                                                        .id("line-height-plus")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, e, w, cx| {
                                                                this.adjust_body_line_height(
                                                                    0.05, e, w, cx,
                                                                )
                                                            }),
                                                        )
                                                        .child("+"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .text_size(px(12.0))
                                        .child("Shrunk scale")
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .h(px(24.0))
                                                        .px_2()
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(colors.border)
                                                        .bg(colors.surface_elevated)
                                                        .cursor_pointer()
                                                        .id("shrunk-minus")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, e, w, cx| {
                                                                this.adjust_shrunk_scale(
                                                                    -0.02, e, w, cx,
                                                                )
                                                            }),
                                                        )
                                                        .child("-"),
                                                )
                                                .child(div().text_size(px(12.0)).child(format!(
                                                    "{:.2}",
                                                    self.style_config.shrunk.font_size_scale
                                                )))
                                                .child(
                                                    div()
                                                        .h(px(24.0))
                                                        .px_2()
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(colors.border)
                                                        .bg(colors.surface_elevated)
                                                        .cursor_pointer()
                                                        .id("shrunk-plus")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, e, w, cx| {
                                                                this.adjust_shrunk_scale(
                                                                    0.02, e, w, cx,
                                                                )
                                                            }),
                                                        )
                                                        .child("+"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .text_size(px(12.0))
                                        .child("Highlight color")
                                        .child(
                                            div()
                                                .h(px(24.0))
                                                .px_2()
                                                .rounded_sm()
                                                .border_1()
                                                .border_color(colors.border)
                                                .bg(colors.surface_elevated)
                                                .cursor_pointer()
                                                .id("highlight-cycle")
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(Self::cycle_highlight_color),
                                                )
                                                .child(
                                                    self.style_config.highlight.background.clone(),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(colors.text_muted)
                                        .child(format!(
                                            "Config: {}",
                                            self.style_config_path.display()
                                        )),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(colors.text_muted)
                                        .child(self.generated_css.clone()),
                                ),
                        ),
                )
            })
    }
}

impl DebateEditor {
    pub(super) fn adjust_body_font_size(
        &mut self,
        delta: f32,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.style_mutation(
            |style| style.body.font_size_pt = (style.body.font_size_pt + delta).clamp(8.0, 24.0),
            cx,
        );
    }

    pub(super) fn adjust_body_line_height(
        &mut self,
        delta: f32,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.style_mutation(
            |style| style.body.line_height = (style.body.line_height + delta).clamp(0.8, 2.0),
            cx,
        );
    }

    pub(super) fn adjust_shrunk_scale(
        &mut self,
        delta: f32,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.style_mutation(
            |style| {
                style.shrunk.font_size_scale =
                    (style.shrunk.font_size_scale + delta).clamp(0.5, 1.0)
            },
            cx,
        );
    }

    pub(super) fn cycle_highlight_color(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.style_mutation(
            |style| {
                style.highlight.background = match style.highlight.background.as_str() {
                    "#fff176" => "#ffd54f".to_string(),
                    "#ffd54f" => "#c5e1a5".to_string(),
                    "#c5e1a5" => "#81d4fa".to_string(),
                    _ => "#fff176".to_string(),
                }
            },
            cx,
        );
    }
}
