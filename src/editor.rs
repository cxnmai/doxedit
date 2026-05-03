use crate::{
    db8_document::{Db8Block, Db8Document, Db8Span, StyleSet},
    project_tree::{ProjectTree, TreeEntry},
    theme::{AppColors, ColorMode},
};
use gpui::{
    AnyElement, ClipboardItem, Context, FocusHandle, IntoElement, KeystrokeEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathPromptOptions, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub struct DebateEditor {
    color_mode: ColorMode,
    document_view_mode: DocumentViewMode,
    settings_open: bool,
    sidebar_open: bool,
    project_tree: ProjectTree,
    selected_file: Option<PathBuf>,
    opened_document: Option<OpenedDocument>,
    expanded_folders: HashSet<PathBuf>,
    raw_editor_focus: FocusHandle,
    raw_cursor: RawCursor,
    raw_selection: Option<RawSelection>,
    raw_mouse_selecting: bool,
    raw_mouse_anchor: Option<usize>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DocumentViewMode {
    Raw,
    Render,
}

impl DocumentViewMode {
    fn id(self) -> usize {
        match self {
            Self::Raw => 0,
            Self::Render => 1,
        }
    }
}

struct OpenedDocument {
    path: PathBuf,
    raw: String,
    parsed: Db8Document,
}

#[derive(Clone, Copy, Default)]
struct RawCursor {
    line: usize,
    column: usize,
}

#[derive(Clone, Copy)]
struct RawSelection {
    anchor: usize,
    head: usize,
}

impl RawSelection {
    fn ordered(self) -> (usize, usize) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    fn is_empty(self) -> bool {
        self.anchor == self.head
    }
}

impl DebateEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_tree = ProjectTree::load(project_root);
        let expanded_folders = project_tree.folder_paths();
        let raw_editor_focus = cx.focus_handle();
        let keyboard_subscription = cx.observe_keystrokes(Self::handle_raw_editor_keystroke);

        Self {
            color_mode: ColorMode::initial(),
            document_view_mode: DocumentViewMode::Raw,
            settings_open: false,
            sidebar_open: true,
            project_tree,
            selected_file: None,
            opened_document: None,
            expanded_folders,
            raw_editor_focus,
            raw_cursor: RawCursor::default(),
            raw_selection: None,
            raw_mouse_selecting: false,
            raw_mouse_anchor: None,
            _subscriptions: vec![keyboard_subscription],
        }
    }

    fn toggle_sidebar(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_open = !self.sidebar_open;
        cx.notify();
    }

    fn toggle_settings(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_open = !self.settings_open;
        cx.notify();
    }

    fn toggle_color_mode(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.color_mode = self.color_mode.toggled();
        cx.notify();
    }

    fn open_project_directory(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open".into()),
        });

        cx.spawn(async move |this, cx| {
            let selected_path = match paths.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                _ => None,
            };

            if let Some(project_root) = selected_path {
                let project_tree = ProjectTree::load(project_root);
                let expanded_folders = project_tree.folder_paths();

                let _ = this.update(cx, |this, cx| {
                    this.project_tree = project_tree;
                    this.selected_file = None;
                    this.opened_document = None;
                    this.raw_cursor = RawCursor::default();
                    this.raw_selection = None;
                    this.raw_mouse_selecting = false;
                    this.raw_mouse_anchor = None;
                    this.expanded_folders = expanded_folders;
                    this.sidebar_open = true;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn select_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|_| String::new());
        let parsed = Db8Document::parse(&raw);

        self.selected_file = Some(path.clone());
        self.opened_document = Some(OpenedDocument { path, raw, parsed });
        self.raw_cursor = RawCursor::default();
        self.raw_selection = None;
        self.raw_mouse_selecting = false;
        self.raw_mouse_anchor = None;
        cx.notify();
    }

    fn set_document_view_mode(&mut self, mode: DocumentViewMode, cx: &mut Context<Self>) {
        self.document_view_mode = mode;
        cx.notify();
    }

    fn start_raw_document_mouse_selection(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let end = raw_document_mouse_offset(&document.raw, f32::from(event.position.y));
        self.raw_mouse_selecting = false;
        self.raw_mouse_anchor = Some(end);
        self.raw_selection = None;
        self.raw_cursor = raw_offset_to_cursor(&document.raw, end);
        window.focus(&self.raw_editor_focus);
        cx.notify();
    }

    fn update_raw_document_mouse_selection(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(anchor) = self.raw_mouse_anchor else {
            return;
        };

        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let head = raw_document_mouse_offset(&document.raw, f32::from(event.position.y));
        if anchor == head {
            return;
        }

        self.raw_mouse_selecting = true;
        self.raw_selection = Some(RawSelection { anchor, head });
        self.raw_cursor = raw_offset_to_cursor(&document.raw, head);
        cx.notify();
    }

    fn start_raw_mouse_selection(
        &mut self,
        line: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let anchor = raw_line_mouse_offset(
            &document.raw,
            line,
            f32::from(event.position.x),
            self.sidebar_open,
        );

        self.raw_mouse_selecting = false;
        self.raw_mouse_anchor = Some(anchor);
        self.raw_selection = None;
        self.raw_cursor = raw_offset_to_cursor(&document.raw, anchor);
        window.focus(&self.raw_editor_focus);
        cx.notify();
    }

    fn update_raw_mouse_selection(
        &mut self,
        line: usize,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(anchor) = self.raw_mouse_anchor else {
            return;
        };

        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let head = raw_line_mouse_offset(
            &document.raw,
            line,
            f32::from(event.position.x),
            self.sidebar_open,
        );
        if anchor == head {
            return;
        }

        self.raw_mouse_selecting = true;
        self.raw_selection = Some(RawSelection { anchor, head });
        self.raw_cursor = raw_offset_to_cursor(&document.raw, head);
        cx.notify();
    }

    fn finish_raw_mouse_selection(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.raw_mouse_selecting {
            self.raw_mouse_selecting = false;
            cx.notify();
        }
        self.raw_mouse_anchor = None;
    }

    fn toggle_folder(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if !self.expanded_folders.remove(&path) {
            self.expanded_folders.insert(path);
        }

        cx.notify();
    }

    fn handle_raw_editor_keystroke(
        &mut self,
        event: &KeystrokeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.document_view_mode != DocumentViewMode::Raw
            || !self.raw_editor_focus.is_focused(window)
            || self.opened_document.is_none()
        {
            return;
        }

        let keystroke = &event.keystroke;

        if keystroke.modifiers.control || keystroke.modifiers.platform {
            match keystroke.key.as_str() {
                "a" => self.raw_select_all(),
                "c" => self.raw_copy_selection_or_line(cx),
                "v" => {
                    if self.raw_paste_clipboard(cx) {
                        self.autosave_raw_document();
                    }
                }
                _ => {}
            }
            cx.notify();
            return;
        }

        let edited = match keystroke.key.as_str() {
            "backspace" => self.raw_backspace(),
            "delete" => self.raw_delete(),
            "enter" => self.raw_insert_newline(),
            "left" => {
                self.raw_move_left(keystroke.modifiers.shift);
                false
            }
            "right" => {
                self.raw_move_right(keystroke.modifiers.shift);
                false
            }
            "up" => {
                self.raw_move_up(keystroke.modifiers.shift);
                false
            }
            "down" => {
                self.raw_move_down(keystroke.modifiers.shift);
                false
            }
            _ => {
                if let Some(text) = keystroke.key_char.as_ref() {
                    if text.chars().all(|ch| !ch.is_control()) {
                        self.raw_insert_text(text)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };

        if edited {
            self.autosave_raw_document();
        }

        cx.notify();
    }

    fn raw_lines(&self) -> Vec<String> {
        self.opened_document
            .as_ref()
            .map(|document| split_raw_lines(&document.raw))
            .unwrap_or_default()
    }

    fn update_raw_lines(&mut self, lines: Vec<String>) {
        if let Some(document) = self.opened_document.as_mut() {
            document.raw = lines.join("\n");
            document.parsed = Db8Document::parse(&document.raw);
        }
    }

    fn raw_insert_text(&mut self, text: &str) -> bool {
        self.raw_delete_selection();
        let mut lines = self.raw_lines();
        ensure_line(&mut lines, self.raw_cursor.line);
        let line = &mut lines[self.raw_cursor.line];
        let column = clamp_to_char_boundary(line, self.raw_cursor.column);
        line.insert_str(column, text);
        self.raw_cursor.column = column + text.len();
        self.update_raw_lines(lines);
        true
    }

    fn raw_select_all(&mut self) {
        if let Some(document) = self.opened_document.as_ref() {
            let end = document.raw.len();
            self.raw_selection = Some(RawSelection {
                anchor: 0,
                head: end,
            });
            self.raw_cursor = raw_offset_to_cursor(&document.raw, end);
        }
    }

    fn raw_copy_selection_or_line(&self, cx: &mut Context<Self>) {
        if let Some(selected_text) = self.raw_selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
        } else if let Some(line) = self.raw_lines().get(self.raw_cursor.line) {
            cx.write_to_clipboard(ClipboardItem::new_string(line.clone()));
        }
    }

    fn raw_paste_clipboard(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return false;
        };

        self.raw_delete_selection();
        let mut changed = false;
        let mut pasted_lines = text.split('\n').peekable();

        while let Some(line) = pasted_lines.next() {
            if !line.is_empty() {
                changed |= self.raw_insert_text(line);
            }

            if pasted_lines.peek().is_some() {
                changed |= self.raw_insert_newline();
            }
        }

        changed
    }

    fn raw_insert_newline(&mut self) -> bool {
        self.raw_delete_selection();
        let mut lines = self.raw_lines();
        ensure_line(&mut lines, self.raw_cursor.line);
        let line = &mut lines[self.raw_cursor.line];
        let column = clamp_to_char_boundary(line, self.raw_cursor.column);
        let remainder = line.split_off(column);
        lines.insert(self.raw_cursor.line + 1, remainder);
        self.raw_cursor.line += 1;
        self.raw_cursor.column = 0;
        self.update_raw_lines(lines);
        true
    }

    fn raw_backspace(&mut self) -> bool {
        if self.raw_delete_selection() {
            return true;
        }

        let mut lines = self.raw_lines();
        ensure_line(&mut lines, self.raw_cursor.line);

        if self.raw_cursor.column > 0 {
            let line = &mut lines[self.raw_cursor.line];
            let column = clamp_to_char_boundary(line, self.raw_cursor.column);
            let previous_column = previous_char_boundary(line, column);
            line.replace_range(previous_column..column, "");
            self.raw_cursor.column = previous_column;
        } else if self.raw_cursor.line > 0 {
            let removed = lines.remove(self.raw_cursor.line);
            self.raw_cursor.line -= 1;
            self.raw_cursor.column = lines[self.raw_cursor.line].len();
            lines[self.raw_cursor.line].push_str(&removed);
        } else {
            return false;
        }

        self.update_raw_lines(lines);
        true
    }

    fn raw_delete(&mut self) -> bool {
        if self.raw_delete_selection() {
            return true;
        }

        let mut lines = self.raw_lines();
        ensure_line(&mut lines, self.raw_cursor.line);
        let line_len = lines[self.raw_cursor.line].len();

        if self.raw_cursor.column < line_len {
            let line = &mut lines[self.raw_cursor.line];
            let column = clamp_to_char_boundary(line, self.raw_cursor.column);
            let next_column = next_char_boundary(line, column);
            line.replace_range(column..next_column, "");
        } else if self.raw_cursor.line + 1 < lines.len() {
            let next = lines.remove(self.raw_cursor.line + 1);
            lines[self.raw_cursor.line].push_str(&next);
        } else {
            return false;
        }

        self.update_raw_lines(lines);
        true
    }

    fn raw_move_left(&mut self, selecting: bool) {
        let old_offset = self.raw_cursor_offset();
        if self.raw_cursor.column > 0 {
            if let Some(line) = self.raw_lines().get(self.raw_cursor.line) {
                self.raw_cursor.column = previous_char_boundary(line, self.raw_cursor.column);
            }
        } else if self.raw_cursor.line > 0 {
            self.raw_cursor.line -= 1;
            self.raw_cursor.column = self
                .raw_lines()
                .get(self.raw_cursor.line)
                .map_or(0, String::len);
        }
        self.update_raw_selection_after_move(old_offset, selecting);
    }

    fn raw_move_right(&mut self, selecting: bool) {
        let old_offset = self.raw_cursor_offset();
        let lines = self.raw_lines();
        let line_len = lines.get(self.raw_cursor.line).map_or(0, String::len);

        if self.raw_cursor.column < line_len {
            if let Some(line) = lines.get(self.raw_cursor.line) {
                self.raw_cursor.column = next_char_boundary(line, self.raw_cursor.column);
            }
        } else if self.raw_cursor.line + 1 < lines.len() {
            self.raw_cursor.line += 1;
            self.raw_cursor.column = 0;
        }
        self.update_raw_selection_after_move(old_offset, selecting);
    }

    fn raw_move_up(&mut self, selecting: bool) {
        let old_offset = self.raw_cursor_offset();
        if self.raw_cursor.line > 0 {
            self.raw_cursor.line -= 1;
            self.clamp_raw_cursor_column();
        }
        self.update_raw_selection_after_move(old_offset, selecting);
    }

    fn raw_move_down(&mut self, selecting: bool) {
        let old_offset = self.raw_cursor_offset();
        let lines = self.raw_lines();
        if self.raw_cursor.line + 1 < lines.len() {
            self.raw_cursor.line += 1;
            self.clamp_raw_cursor_column();
        }
        self.update_raw_selection_after_move(old_offset, selecting);
    }

    fn clamp_raw_cursor_column(&mut self) {
        let line_len = self
            .raw_lines()
            .get(self.raw_cursor.line)
            .map_or(0, String::len);
        self.raw_cursor.column = self.raw_cursor.column.min(line_len);
    }

    fn autosave_raw_document(&self) {
        if let Some(document) = self.opened_document.as_ref() {
            let _ = std::fs::write(&document.path, &document.raw);
        }
    }

    fn raw_cursor_offset(&self) -> usize {
        self.opened_document.as_ref().map_or(0, |document| {
            raw_cursor_to_offset(&document.raw, self.raw_cursor)
        })
    }

    fn update_raw_selection_after_move(&mut self, old_offset: usize, selecting: bool) {
        if selecting {
            let head = self.raw_cursor_offset();
            let anchor = self
                .raw_selection
                .map_or(old_offset, |selection| selection.anchor);
            self.raw_selection = Some(RawSelection { anchor, head });
        } else {
            self.raw_selection = None;
        }
    }

    fn raw_selected_text(&self) -> Option<String> {
        let document = self.opened_document.as_ref()?;
        let selection = self.raw_selection?;
        let (start, end) = selection.ordered();

        if start == end {
            None
        } else {
            Some(document.raw[start..end].to_string())
        }
    }

    fn raw_delete_selection(&mut self) -> bool {
        let Some(selection) = self.raw_selection.take() else {
            return false;
        };
        let (start, end) = selection.ordered();

        if start == end {
            return false;
        }

        if let Some(document) = self.opened_document.as_mut() {
            document.raw.replace_range(start..end, "");
            document.parsed = Db8Document::parse(&document.raw);
            self.raw_cursor = raw_offset_to_cursor(&document.raw, start);
            true
        } else {
            false
        }
    }

    fn render_project_tree(&self, colors: AppColors, cx: &mut Context<Self>) -> impl IntoElement {
        let root_name = self.project_tree.root_name();
        let root_path = self.project_tree.root.clone();

        div()
            .w(px(240.0))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(colors.surface)
            .border_r_1()
            .border_color(colors.border)
            .child(
                div()
                    .h(px(40.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_3()
                    .border_b_1()
                    .border_color(colors.border)
                    .text_size(px(12.0))
                    .text_color(colors.text_muted)
                    .child(
                        div()
                            .overflow_hidden()
                            .text_overflow(gpui::TextOverflow::Truncate("".into()))
                            .child(root_name),
                    )
                    .child(
                        div()
                            .id("open-project-directory")
                            .flex_none()
                            .h(px(24.0))
                            .px_2()
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
                            .on_click(cx.listener(Self::open_project_directory))
                            .child("Open"),
                    ),
            )
            .child(
                div()
                    .id("project-tree-scroll")
                    .flex_1()
                    .overflow_scroll()
                    .py_2()
                    .when(self.project_tree.entries.is_empty(), |this| {
                        this.child(
                            div()
                                .px_3()
                                .py_2()
                                .text_size(px(12.0))
                                .text_color(colors.text_muted)
                                .child("No .db8 files"),
                        )
                    })
                    .children(
                        self.project_tree
                            .entries
                            .iter()
                            .map(|entry| self.render_tree_entry(entry, 0, &root_path, colors, cx)),
                    ),
            )
    }

    fn render_tree_entry(
        &self,
        entry: &TreeEntry,
        depth: usize,
        root: &Path,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let indent = px(10.0 + depth as f32 * 14.0);

        match entry {
            TreeEntry::Folder {
                name,
                path,
                children,
            } => {
                let expanded = self.expanded_folders.contains(path);
                let path_for_click = path.clone();
                let display_path = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id(SharedString::from(format!("folder-{display_path}")))
                            .h(px(24.0))
                            .flex()
                            .items_center()
                            .gap_1()
                            .pl(indent)
                            .pr_2()
                            .text_size(px(12.0))
                            .text_color(colors.text_muted)
                            .cursor_pointer()
                            .hover(|this| this.bg(colors.surface_elevated))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_folder(path_for_click.clone(), cx);
                            }))
                            .child(if expanded { "▾" } else { "▸" })
                            .child(name.clone()),
                    )
                    .when(expanded, |this| {
                        this.children(children.iter().map(|child| {
                            self.render_tree_entry(child, depth + 1, root, colors, cx)
                        }))
                    })
                    .into_any_element()
            }
            TreeEntry::File { name, path } => {
                let selected = self.selected_file.as_ref() == Some(path);
                let path_for_click = path.clone();
                let display_path = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                div()
                    .id(SharedString::from(display_path))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .gap_2()
                    .pl(indent)
                    .pr_2()
                    .text_size(px(12.0))
                    .text_color(colors.text)
                    .cursor_pointer()
                    .when(selected, |this| this.bg(colors.surface_elevated))
                    .hover(|this| this.bg(colors.surface_elevated))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_file(path_for_click.clone(), cx);
                    }))
                    .child(div().size(px(6.0)).rounded_sm().bg(if selected {
                        colors.accent
                    } else {
                        colors.border
                    }))
                    .child(name.clone())
                    .into_any_element()
            }
        }
    }

    fn render_document_pane(&self, colors: AppColors, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .child(self.render_document_toolbar(colors, cx))
            .child(
                div()
                    .id("document-scroll")
                    .flex_1()
                    .overflow_scroll()
                    .when_some(self.opened_document.as_ref(), |this, document| {
                        match self.document_view_mode {
                            DocumentViewMode::Raw => {
                                this.child(self.render_raw_document(document, colors, cx))
                            }
                            DocumentViewMode::Render => {
                                this.child(self.render_db8_document(&document.parsed, colors))
                            }
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
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .rounded_sm()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.surface_elevated)
                    .child(self.render_view_mode_button("Raw", DocumentViewMode::Raw, colors, cx))
                    .child(self.render_view_mode_button(
                        "Render",
                        DocumentViewMode::Render,
                        colors,
                        cx,
                    )),
            )
    }

    fn render_view_mode_button(
        &self,
        label: &'static str,
        mode: DocumentViewMode,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.document_view_mode == mode;

        div()
            .id(("view-mode", mode.id()))
            .h(px(26.0))
            .px_3()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(12.0))
            .text_color(if selected {
                colors.text
            } else {
                colors.text_muted
            })
            .cursor_pointer()
            .when(selected, |this| this.bg(colors.background))
            .hover(|this| this.bg(colors.background))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_document_view_mode(mode, cx);
            }))
            .child(label)
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

    fn render_db8_document(&self, document: &Db8Document, colors: AppColors) -> impl IntoElement {
        div().p_4().flex().flex_col().gap_2().children(
            document
                .blocks
                .iter()
                .map(|block| self.render_db8_block(block, colors)),
        )
    }

    fn render_db8_block(&self, block: &Db8Block, colors: AppColors) -> impl IntoElement {
        div()
            .id(("render-line", block.source_line))
            .min_h(px(22.0))
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1()
            .children(
                block
                    .spans
                    .iter()
                    .map(|span| self.render_db8_span(span, colors)),
            )
    }

    fn render_db8_span(&self, span: &Db8Span, colors: AppColors) -> AnyElement {
        let styles = span.styles;

        div()
            .px(if styles.highlight || styles.tag || styles.cite {
                px(3.0)
            } else {
                px(0.0)
            })
            .py(if styles.highlight || styles.tag || styles.cite {
                px(1.0)
            } else {
                px(0.0)
            })
            .rounded_sm()
            .text_size(if styles.small { px(11.0) } else { px(13.0) })
            .text_color(span_text_color(styles, colors))
            .when(styles.bold || styles.pocket || styles.hat, |this| {
                this.font_weight(gpui::FontWeight::BOLD)
            })
            .when(styles.underline, |this| this.underline())
            .when(styles.highlight, |this| {
                this.bg(gpui::yellow().opacity(0.35))
            })
            .when(styles.pocket, |this| this.bg(gpui::blue().opacity(0.12)))
            .when(styles.hat, |this| this.text_size(px(15.0)))
            .when(styles.block, |this| {
                this.border_l_2().border_color(colors.accent).pl_2()
            })
            .when(styles.tag, |this| {
                this.bg(colors.surface_elevated)
                    .border_1()
                    .border_color(colors.border)
                    .text_size(px(11.0))
            })
            .when(styles.cite, |this| {
                this.italic()
                    .bg(colors.surface_elevated)
                    .text_color(colors.text_muted)
            })
            .child(span.text.clone())
            .into_any_element()
    }
}

fn span_text_color(styles: StyleSet, colors: AppColors) -> gpui::Rgba {
    if styles.tag || styles.cite {
        colors.text_muted
    } else {
        colors.text
    }
}

fn split_raw_lines(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        vec![String::new()]
    } else {
        raw.split('\n').map(str::to_string).collect()
    }
}

fn raw_cursor_to_offset(raw: &str, cursor: RawCursor) -> usize {
    let mut offset = 0;

    for (line_index, line) in raw.split('\n').enumerate() {
        if line_index == cursor.line {
            return offset + cursor.column.min(line.len());
        }

        offset += line.len() + 1;
    }

    raw.len()
}

fn raw_offset_to_cursor(raw: &str, offset: usize) -> RawCursor {
    let mut line_start = 0;

    for (line_index, line) in raw.split('\n').enumerate() {
        let line_end = line_start + line.len();

        if offset <= line_end {
            return RawCursor {
                line: line_index,
                column: offset.saturating_sub(line_start),
            };
        }

        line_start = line_end + 1;
    }

    RawCursor {
        line: raw.split('\n').count().saturating_sub(1),
        column: raw.rsplit('\n').next().map_or(0, str::len),
    }
}

fn clamp_to_char_boundary(text: &str, column: usize) -> usize {
    let mut column = column.min(text.len());
    while column > 0 && !text.is_char_boundary(column) {
        column -= 1;
    }
    column
}

fn previous_char_boundary(text: &str, column: usize) -> usize {
    let column = clamp_to_char_boundary(text, column);
    text[..column]
        .char_indices()
        .last()
        .map_or(0, |(index, _)| index)
}

fn next_char_boundary(text: &str, column: usize) -> usize {
    let column = clamp_to_char_boundary(text, column);
    if column >= text.len() {
        text.len()
    } else {
        text[column..]
            .char_indices()
            .nth(1)
            .map_or(text.len(), |(offset, _)| column + offset)
    }
}

fn raw_line_start_offset(raw: &str, target_line: usize) -> usize {
    let mut offset = 0;

    for (line_index, line) in raw.split('\n').enumerate() {
        if line_index == target_line {
            return offset;
        }

        offset += line.len() + 1;
    }

    raw.len()
}

fn raw_line_mouse_offset(raw: &str, target_line: usize, mouse_x: f32, sidebar_open: bool) -> usize {
    const RAW_EDITOR_PADDING_X: f32 = 16.0;
    const RAW_LINE_NUMBER_WIDTH: f32 = 48.0;
    const SIDEBAR_WIDTH: f32 = 240.0;

    let document_left = if sidebar_open { SIDEBAR_WIDTH } else { 0.0 };
    let text_left = document_left + RAW_EDITOR_PADDING_X + RAW_LINE_NUMBER_WIDTH;
    let target_char = ((mouse_x - text_left) / RAW_EDITOR_CHAR_WIDTH)
        .round()
        .max(0.0) as usize;
    let line_start = raw_line_start_offset(raw, target_line);
    let line = raw.split('\n').nth(target_line).unwrap_or("");
    let line_char_count = line.chars().count();
    let column = char_index_to_byte_offset(line, target_char.min(line_char_count));

    line_start + column
}

fn raw_document_mouse_offset(raw: &str, mouse_y: f32) -> usize {
    const APP_HEADER_HEIGHT: f32 = 40.0;
    const DOCUMENT_TOOLBAR_HEIGHT: f32 = 40.0;
    const RAW_EDITOR_PADDING_Y: f32 = 16.0;
    const RAW_LINE_HEIGHT: f32 = 22.0;

    let line_count = raw.split('\n').count().max(1);
    let raw_top = APP_HEADER_HEIGHT + DOCUMENT_TOOLBAR_HEIGHT + RAW_EDITOR_PADDING_Y;
    let relative_y = mouse_y - raw_top;

    if relative_y <= 0.0 {
        return 0;
    }

    let line = (relative_y / RAW_LINE_HEIGHT).floor() as usize;
    if line >= line_count {
        raw.len()
    } else {
        raw_line_start_offset(raw, line)
    }
}

fn render_raw_line_segments(
    line: &str,
    cursor_column: usize,
    show_cursor: bool,
    selected_range: Option<(usize, usize)>,
    colors: AppColors,
) -> Vec<AnyElement> {
    let mut segments = Vec::new();

    if let Some((selection_start, selection_end)) = selected_range {
        push_raw_text_segment(&mut segments, &line[..selection_start], false, colors);
        push_raw_text_segment(
            &mut segments,
            &line[selection_start..selection_end],
            true,
            colors,
        );
        push_raw_text_segment(&mut segments, &line[selection_end..], false, colors);
    } else if show_cursor {
        let cursor_column = clamp_to_char_boundary(line, cursor_column);
        push_raw_text_segment(&mut segments, &line[..cursor_column], false, colors);
        segments.push(raw_cursor_element(colors).into_any_element());
        push_raw_text_segment(&mut segments, &line[cursor_column..], false, colors);
    } else {
        push_raw_text_segment(&mut segments, line, false, colors);
    }

    if segments.is_empty() {
        segments.push(div().child(" ").into_any_element());
    }

    segments
}

const RAW_EDITOR_CHAR_WIDTH: f32 = 7.8;

fn raw_cursor_element(colors: AppColors) -> gpui::Div {
    div().flex_none().w(px(2.0)).h(px(16.0)).bg(colors.accent)
}

fn push_raw_text_segment(
    segments: &mut Vec<AnyElement>,
    text: &str,
    selected: bool,
    _colors: AppColors,
) {
    if text.is_empty() {
        return;
    }

    segments.push(
        div()
            .when(selected, |this| this.bg(gpui::blue().opacity(0.22)))
            .child(text.to_string())
            .into_any_element(),
    );
}

fn char_index_to_byte_offset(text: &str, char_index: usize) -> usize {
    if char_index == 0 {
        0
    } else {
        text.char_indices()
            .nth(char_index)
            .map_or(text.len(), |(offset, _)| offset)
    }
}

fn ensure_line(lines: &mut Vec<String>, line: usize) {
    while lines.len() <= line {
        lines.push(String::new());
    }
}

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
                                ),
                        ),
                )
            })
    }
}
