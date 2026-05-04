use crate::{
    db8_document::{Db8Block, Db8Document, StyleSet},
    db8_style::{
        Db8StyleConfig, db8_document_to_typst, load_or_create_style_config, save_style_config,
        style_config_to_css,
    },
    project_tree::{ProjectTree, TreeEntry},
    theme::{AppColors, ColorMode},
};
use gpui::{
    AnyElement, ClipboardItem, Context, FocusHandle, IntoElement, KeystrokeEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathPromptOptions, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use std::{
    any::Any,
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
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
    render_editor_focus: FocusHandle,
    render_cursor: RawCursor,
    render_selection: Option<RawSelection>,
    render_mouse_selecting: bool,
    render_mouse_anchor: Option<usize>,
    render_active_styles: StyleSet,
    style_config_path: PathBuf,
    style_config: Db8StyleConfig,
    generated_css: String,
    export_status: Option<String>,
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

#[derive(Clone)]
struct DraggedTreeFile {
    path: PathBuf,
    name: SharedString,
}

struct DraggedTreeFilePreview {
    name: SharedString,
}

impl Render for DraggedTreeFilePreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(gpui::blue())
            .bg(gpui::blue().opacity(0.2))
            .text_size(px(12.0))
            .child(self.name.clone())
    }
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
        let project_tree = ProjectTree::load(project_root.clone());
        let expanded_folders = project_tree.folder_paths();
        let raw_editor_focus = cx.focus_handle();
        let render_editor_focus = cx.focus_handle();
        let keyboard_subscription = cx.observe_keystrokes(Self::handle_editor_keystroke);
        let style_config_path = project_root.join(".db8_style.json");
        let style_config = load_or_create_style_config(&style_config_path);
        let generated_css = style_config_to_css(&style_config);

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
            render_editor_focus,
            render_cursor: RawCursor::default(),
            render_selection: None,
            render_mouse_selecting: false,
            render_mouse_anchor: None,
            render_active_styles: StyleSet::default(),
            style_config_path,
            style_config,
            generated_css,
            export_status: None,
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
                let project_tree = ProjectTree::load(project_root.clone());
                let expanded_folders = project_tree.folder_paths();
                let style_config_path = project_root.join(".db8_style.json");
                let style_config = load_or_create_style_config(&style_config_path);
                let generated_css = style_config_to_css(&style_config);

                let _ = this.update(cx, |this, cx| {
                    this.project_tree = project_tree;
                    this.selected_file = None;
                    this.opened_document = None;
                    this.raw_cursor = RawCursor::default();
                    this.raw_selection = None;
                    this.raw_mouse_selecting = false;
                    this.raw_mouse_anchor = None;
                    this.render_cursor = RawCursor::default();
                    this.render_selection = None;
                    this.render_mouse_selecting = false;
                    this.render_mouse_anchor = None;
                    this.render_active_styles = StyleSet::default();
                    this.style_config_path = style_config_path;
                    this.style_config = style_config;
                    this.generated_css = generated_css;
                    this.export_status = None;
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
        self.render_cursor = RawCursor::default();
        self.render_selection = None;
        self.render_mouse_selecting = false;
        self.render_mouse_anchor = None;
        self.render_active_styles = StyleSet::default();
        self.export_status = None;
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

    fn refresh_project_tree(&mut self, cx: &mut Context<Self>) {
        self.project_tree = ProjectTree::load(self.project_tree.root.clone());
        let folder_paths = self.project_tree.folder_paths();
        self.expanded_folders
            .retain(|path| folder_paths.contains(path));
        self.expanded_folders.extend(folder_paths);
        cx.notify();
    }

    fn create_root_db8_file(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let base_name = "new_file";
        let mut index = 0;

        let file_path = loop {
            let suffix = if index == 0 {
                String::new()
            } else {
                format!("_{}", index)
            };
            let candidate = self
                .project_tree
                .root
                .join(format!("{}{}.db8", base_name, suffix));
            if !candidate.exists() {
                break candidate;
            }
            index += 1;
        };

        if std::fs::write(&file_path, "").is_ok() {
            self.refresh_project_tree(cx);
            self.select_file(file_path, cx);
        }
    }

    fn can_drop_tree_file_to(
        dragged_value: &dyn Any,
        target_folder: &Path,
        _window: &mut Window,
        _cx: &mut gpui::App,
    ) -> bool {
        let Some(dragged_file) = dragged_value.downcast_ref::<DraggedTreeFile>() else {
            return false;
        };

        if !dragged_file.path.exists() {
            return false;
        }

        let Some(file_name) = dragged_file.path.file_name() else {
            return false;
        };

        let destination = target_folder.join(file_name);
        destination != dragged_file.path && !destination.exists()
    }

    fn move_file_to_directory(
        &mut self,
        dragged_file: &DraggedTreeFile,
        target_folder: &Path,
        cx: &mut Context<Self>,
    ) {
        let Some(file_name) = dragged_file.path.file_name() else {
            return;
        };

        let destination = target_folder.join(file_name);
        if destination == dragged_file.path || destination.exists() {
            return;
        }

        if std::fs::rename(&dragged_file.path, &destination).is_err() {
            return;
        }

        if self.selected_file.as_ref() == Some(&dragged_file.path) {
            self.selected_file = Some(destination.clone());
        }

        if let Some(document) = self.opened_document.as_mut()
            && document.path == dragged_file.path
        {
            document.path = destination;
        }

        self.refresh_project_tree(cx);
    }

    fn handle_editor_keystroke(
        &mut self,
        event: &KeystrokeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.opened_document.is_none() {
            return;
        }

        let editing_raw = self.document_view_mode == DocumentViewMode::Raw
            && self.raw_editor_focus.is_focused(window);
        let editing_render = self.document_view_mode == DocumentViewMode::Render
            && self.render_editor_focus.is_focused(window);

        if !editing_raw && !editing_render {
            return;
        }

        let keystroke = &event.keystroke;

        if keystroke.modifiers.control || keystroke.modifiers.platform {
            match keystroke.key.as_str() {
                "a" => {
                    if editing_raw {
                        self.raw_select_all();
                    } else {
                        self.render_select_all();
                    }
                }
                "c" => {
                    if editing_raw {
                        self.raw_copy_selection_or_line(cx);
                    } else {
                        self.render_copy_selection_or_line(cx);
                    }
                }
                "v" => {
                    let changed = if editing_raw {
                        self.raw_paste_clipboard(cx)
                    } else {
                        self.render_paste_clipboard(cx)
                    };
                    if changed {
                        self.autosave_raw_document();
                    }
                }
                _ => {}
            }
            cx.notify();
            return;
        }

        let edited = if editing_raw {
            match keystroke.key.as_str() {
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
            }
        } else {
            match keystroke.key.as_str() {
                "backspace" => self.render_backspace(),
                "delete" => self.render_delete(),
                "enter" => self.render_insert_newline(),
                "left" => {
                    self.render_move_left(keystroke.modifiers.shift);
                    false
                }
                "right" => {
                    self.render_move_right(keystroke.modifiers.shift);
                    false
                }
                "up" => {
                    self.render_move_up(keystroke.modifiers.shift);
                    false
                }
                "down" => {
                    self.render_move_down(keystroke.modifiers.shift);
                    false
                }
                _ => {
                    if let Some(text) = keystroke.key_char.as_ref() {
                        if text.chars().all(|ch| !ch.is_control()) {
                            self.render_insert_text(text)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
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

    fn render_select_all(&mut self) {
        if let Some(document) = self.opened_document.as_ref() {
            let line_count = document.parsed.blocks.len().saturating_sub(1);
            let last_line_text = document
                .parsed
                .blocks
                .last()
                .map(rendered_line_text)
                .unwrap_or_default();
            self.render_selection = Some(RawSelection {
                anchor: 0,
                head: rendered_cursor_to_global_offset(
                    &document.parsed,
                    RawCursor {
                        line: line_count,
                        column: last_line_text.len(),
                    },
                ),
            });
            self.render_cursor = RawCursor {
                line: line_count,
                column: last_line_text.len(),
            };
        }
    }

    fn render_copy_selection_or_line(&self, cx: &mut Context<Self>) {
        if let Some(text) = self.render_selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            return;
        }

        if let Some(document) = self.opened_document.as_ref()
            && let Some(block) = document.parsed.blocks.get(self.render_cursor.line)
        {
            cx.write_to_clipboard(ClipboardItem::new_string(rendered_line_text(block)));
        }
    }

    fn render_paste_clipboard(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return false;
        };
        self.render_insert_text(&text)
    }

    fn render_selected_text(&self) -> Option<String> {
        let document = self.opened_document.as_ref()?;
        let selection = self.render_selection?;
        let (start, end) = selection.ordered();
        if start == end {
            return None;
        }

        let mut out = String::new();
        let mut offset = 0usize;

        for block in &document.parsed.blocks {
            let line = rendered_line_text(block);
            let line_end = offset + line.len();
            if end > offset && start < line_end {
                let local_start = start.saturating_sub(offset).min(line.len());
                let local_end = end.saturating_sub(offset).min(line.len());
                if local_start < local_end {
                    out.push_str(&line[local_start..local_end]);
                }
            }
            offset = line_end + 1;
            if end > line_end && start < offset {
                out.push('\n');
            }
        }

        if out.is_empty() { None } else { Some(out) }
    }

    fn render_insert_text(&mut self, text: &str) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        let selection_range = self.render_selection.take().map(RawSelection::ordered);
        let mut raw = document.raw.clone();
        let start_raw = if let Some((start, _)) = selection_range {
            rendered_global_offset_to_raw_offset(&raw, start)
        } else {
            rendered_cursor_to_raw_offset(&raw, self.render_cursor)
        };
        let end_raw = if let Some((_, end)) = selection_range {
            rendered_global_offset_to_raw_offset(&raw, end)
        } else {
            start_raw
        };

        let active_tokens = self.render_active_styles.tokens();
        let replacement = if active_tokens.is_empty() {
            text.to_string()
        } else {
            format!("[{}: {}]", active_tokens.join(" "), text)
        };

        raw.replace_range(start_raw..end_raw, &replacement);
        document.raw = raw;
        document.parsed = Db8Document::parse(&document.raw);

        let new_raw_offset = start_raw + replacement.len();
        self.render_cursor = raw_offset_to_rendered_cursor(&document.raw, new_raw_offset);
        true
    }

    fn render_backspace(&mut self) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        let mut raw = document.raw.clone();

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            let start_raw = rendered_global_offset_to_raw_offset(&raw, start);
            let end_raw = rendered_global_offset_to_raw_offset(&raw, end);
            if start_raw < end_raw {
                raw.replace_range(start_raw..end_raw, "");
                document.raw = raw;
                document.parsed = Db8Document::parse(&document.raw);
                self.render_cursor = raw_offset_to_rendered_cursor(&document.raw, start_raw);
                return true;
            }
        }

        let raw_offset = rendered_cursor_to_raw_offset(&raw, self.render_cursor);
        if raw_offset == 0 {
            return false;
        }

        let previous = previous_char_boundary(&raw, raw_offset);
        raw.replace_range(previous..raw_offset, "");
        document.raw = raw;
        document.parsed = Db8Document::parse(&document.raw);
        self.render_cursor = raw_offset_to_rendered_cursor(&document.raw, previous);
        true
    }

    fn render_delete(&mut self) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        let mut raw = document.raw.clone();

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            let start_raw = rendered_global_offset_to_raw_offset(&raw, start);
            let end_raw = rendered_global_offset_to_raw_offset(&raw, end);
            if start_raw < end_raw {
                raw.replace_range(start_raw..end_raw, "");
                document.raw = raw;
                document.parsed = Db8Document::parse(&document.raw);
                self.render_cursor = raw_offset_to_rendered_cursor(&document.raw, start_raw);
                return true;
            }
        }

        let raw_offset = rendered_cursor_to_raw_offset(&raw, self.render_cursor);
        if raw_offset >= raw.len() {
            return false;
        }

        let next = next_char_boundary(&raw, raw_offset);
        raw.replace_range(raw_offset..next, "");
        document.raw = raw;
        document.parsed = Db8Document::parse(&document.raw);
        self.render_cursor = raw_offset_to_rendered_cursor(&document.raw, raw_offset);
        true
    }

    fn render_insert_newline(&mut self) -> bool {
        self.render_insert_text("\n")
    }

    fn render_move_left(&mut self, selecting: bool) {
        let old_offset = self.render_cursor_offset();
        if self.render_cursor.column > 0 {
            self.render_cursor.column = self.render_cursor.column.saturating_sub(1);
        } else if self.render_cursor.line > 0 {
            self.render_cursor.line -= 1;
            self.render_cursor.column = self
                .opened_document
                .as_ref()
                .and_then(|document| document.parsed.blocks.get(self.render_cursor.line))
                .map(rendered_line_text)
                .map_or(0, |line| line.len());
        }
        self.update_render_selection_after_move(old_offset, selecting);
    }

    fn render_move_right(&mut self, selecting: bool) {
        let old_offset = self.render_cursor_offset();
        if let Some(document) = self.opened_document.as_ref() {
            let line_len = document
                .parsed
                .blocks
                .get(self.render_cursor.line)
                .map(rendered_line_text)
                .map_or(0, |line| line.len());
            if self.render_cursor.column < line_len {
                self.render_cursor.column += 1;
            } else if self.render_cursor.line + 1 < document.parsed.blocks.len() {
                self.render_cursor.line += 1;
                self.render_cursor.column = 0;
            }
        }
        self.update_render_selection_after_move(old_offset, selecting);
    }

    fn render_move_up(&mut self, selecting: bool) {
        let old_offset = self.render_cursor_offset();
        if self.render_cursor.line > 0 {
            self.render_cursor.line -= 1;
            self.clamp_render_cursor_column();
        }
        self.update_render_selection_after_move(old_offset, selecting);
    }

    fn render_move_down(&mut self, selecting: bool) {
        let old_offset = self.render_cursor_offset();
        if let Some(document) = self.opened_document.as_ref()
            && self.render_cursor.line + 1 < document.parsed.blocks.len()
        {
            self.render_cursor.line += 1;
            self.clamp_render_cursor_column();
        }
        self.update_render_selection_after_move(old_offset, selecting);
    }

    fn clamp_render_cursor_column(&mut self) {
        if let Some(document) = self.opened_document.as_ref() {
            let line_len = document
                .parsed
                .blocks
                .get(self.render_cursor.line)
                .map(rendered_line_text)
                .map_or(0, |line| line.len());
            self.render_cursor.column = self.render_cursor.column.min(line_len);
        }
    }

    fn render_cursor_offset(&self) -> usize {
        self.opened_document.as_ref().map_or(0, |document| {
            rendered_cursor_to_global_offset(&document.parsed, self.render_cursor)
        })
    }

    fn update_render_selection_after_move(&mut self, old_offset: usize, selecting: bool) {
        if selecting {
            let head = self.render_cursor_offset();
            let anchor = self
                .render_selection
                .map_or(old_offset, |selection| selection.anchor);
            self.render_selection = Some(RawSelection { anchor, head });
        } else {
            self.render_selection = None;
        }
    }

    fn start_render_mouse_selection(
        &mut self,
        line: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let line_text = document
            .parsed
            .blocks
            .get(line)
            .map(rendered_line_text)
            .unwrap_or_default();
        let line_char_count = line_text.chars().count();
        let column = render_line_mouse_column(f32::from(event.position.x), self.sidebar_open)
            .min(line_char_count);

        self.render_mouse_selecting = false;
        self.render_mouse_anchor = Some(rendered_cursor_to_global_offset(
            &document.parsed,
            RawCursor { line, column },
        ));
        self.render_selection = None;
        self.render_cursor = RawCursor { line, column };
        window.focus(&self.render_editor_focus);
        cx.notify();
    }

    fn update_render_mouse_selection(
        &mut self,
        line: usize,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(anchor) = self.render_mouse_anchor else {
            return;
        };
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let line_text = document
            .parsed
            .blocks
            .get(line)
            .map(rendered_line_text)
            .unwrap_or_default();
        let line_char_count = line_text.chars().count();
        let column = render_line_mouse_column(f32::from(event.position.x), self.sidebar_open)
            .min(line_char_count);
        let head = rendered_cursor_to_global_offset(&document.parsed, RawCursor { line, column });

        if anchor == head {
            return;
        }

        self.render_mouse_selecting = true;
        self.render_selection = Some(RawSelection { anchor, head });
        self.render_cursor = RawCursor { line, column };
        cx.notify();
    }

    fn finish_render_mouse_selection(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.render_mouse_selecting {
            self.render_mouse_selecting = false;
            cx.notify();
        }
        self.render_mouse_anchor = None;
    }

    fn apply_semantic_style_to_selection(&mut self, style_token: &str, cx: &mut Context<Self>) {
        if let Some(selection) = self.render_selection {
            let (start, end) = selection.ordered();
            if start == end {
                self.toggle_active_style(style_token);
                cx.notify();
                return;
            }

            let Some(document) = self.opened_document.as_mut() else {
                return;
            };

            let start_raw = rendered_global_offset_to_raw_offset(&document.raw, start);
            let end_raw = rendered_global_offset_to_raw_offset(&document.raw, end);
            if start_raw >= end_raw || end_raw > document.raw.len() {
                return;
            }

            let selected = document.raw[start_raw..end_raw].to_string();
            let wrapped = format!("[{}: {}]", style_token, selected);
            document.raw.replace_range(start_raw..end_raw, &wrapped);
            document.parsed = Db8Document::parse(&document.raw);
            self.render_selection = None;
            self.render_cursor =
                raw_offset_to_rendered_cursor(&document.raw, start_raw + wrapped.len());
            self.autosave_raw_document();
            cx.notify();
            return;
        }

        self.toggle_active_style(style_token);
        cx.notify();
    }

    fn toggle_active_style(&mut self, style_token: &str) {
        match style_token {
            "pocket" => self.render_active_styles.pocket = !self.render_active_styles.pocket,
            "hat" => self.render_active_styles.hat = !self.render_active_styles.hat,
            "block" => self.render_active_styles.block = !self.render_active_styles.block,
            "tag" => self.render_active_styles.tag = !self.render_active_styles.tag,
            "cite" => self.render_active_styles.cite = !self.render_active_styles.cite,
            "highlight" => {
                self.render_active_styles.highlight = !self.render_active_styles.highlight
            }
            "emphasis" => self.render_active_styles.emphasis = !self.render_active_styles.emphasis,
            "underline" => {
                self.render_active_styles.underline = !self.render_active_styles.underline
            }
            "shrunk" => self.render_active_styles.shrunk = !self.render_active_styles.shrunk,
            _ => {}
        }
    }

    fn style_mutation(
        &mut self,
        mutator: impl FnOnce(&mut Db8StyleConfig),
        cx: &mut Context<Self>,
    ) {
        mutator(&mut self.style_config);
        self.generated_css = style_config_to_css(&self.style_config);
        let _ = save_style_config(&self.style_config_path, &self.style_config);
        cx.notify();
    }

    fn apply_pocket_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("pocket", cx);
    }

    fn apply_hat_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("hat", cx);
    }

    fn apply_block_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("block", cx);
    }

    fn apply_tag_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("tag", cx);
    }

    fn apply_cite_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("cite", cx);
    }

    fn apply_highlight_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("highlight", cx);
    }

    fn apply_emphasis_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("emphasis", cx);
    }

    fn apply_underline_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("underline", cx);
    }

    fn apply_shrunk_style(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_semantic_style_to_selection("shrunk", cx);
    }

    fn export_typst_source(
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

    fn export_pdf(
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

    fn adjust_body_font_size(
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

    fn adjust_body_line_height(
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

    fn adjust_shrunk_scale(
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

    fn cycle_highlight_color(
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
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .id("new-root-db8-file")
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
                                    .on_click(cx.listener(Self::create_root_db8_file))
                                    .child("+ File"),
                            )
                            .child(
                                div()
                                    .id("open-project-directory")
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
                    ),
            )
            .child(
                div()
                    .id("project-tree-scroll")
                    .flex_1()
                    .overflow_scroll()
                    .py_2()
                    .can_drop({
                        let root_path = root_path.clone();
                        move |dragged_value, window, app| {
                            Self::can_drop_tree_file_to(dragged_value, &root_path, window, app)
                        }
                    })
                    .drag_over::<DraggedTreeFile>(move |this, _, _, _| {
                        this.border_1()
                            .border_color(colors.accent)
                            .bg(colors.surface_elevated)
                    })
                    .on_drop(cx.listener({
                        let root_path = root_path.clone();
                        move |this, dragged: &DraggedTreeFile, _, cx| {
                            this.move_file_to_directory(dragged, &root_path, cx);
                        }
                    }))
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
                let path_for_drop_accept = path.clone();
                let path_for_drop_action = path.clone();
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
                            .can_drop(move |dragged_value, window, app| {
                                Self::can_drop_tree_file_to(
                                    dragged_value,
                                    &path_for_drop_accept,
                                    window,
                                    app,
                                )
                            })
                            .drag_over::<DraggedTreeFile>(move |this, _, _, _| {
                                this.bg(colors.surface_elevated)
                                    .border_1()
                                    .border_color(colors.accent)
                            })
                            .on_drop(cx.listener(move |this, dragged: &DraggedTreeFile, _, cx| {
                                this.move_file_to_directory(dragged, &path_for_drop_action, cx);
                            }))
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
                let name_for_drag = name.clone();
                let drag_payload = DraggedTreeFile {
                    path: path.clone(),
                    name: name_for_drag.clone(),
                };
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
                    .cursor_move()
                    .when(selected, |this| this.bg(colors.surface_elevated))
                    .hover(|this| this.bg(colors.surface_elevated))
                    .on_drag(drag_payload, |dragged, _, _, cx| {
                        cx.new(|_| DraggedTreeFilePreview {
                            name: dragged.name.clone(),
                        })
                    })
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
                    .id("document-scroll")
                    .flex_1()
                    .overflow_scroll()
                    .when_some(self.opened_document.as_ref(), |this, document| {
                        match self.document_view_mode {
                            DocumentViewMode::Raw => {
                                this.child(self.render_raw_document(document, colors, cx))
                            }
                            DocumentViewMode::Render => {
                                this.child(self.render_editable_db8_document(document, colors, cx))
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
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.surface_elevated)
                            .child(self.render_view_mode_button(
                                "Raw",
                                DocumentViewMode::Raw,
                                colors,
                                cx,
                            ))
                            .child(self.render_view_mode_button(
                                "Render",
                                DocumentViewMode::Render,
                                colors,
                                cx,
                            )),
                    )
                    .when(
                        self.document_view_mode == DocumentViewMode::Render,
                        |this| {
                            this.child(
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
                                    .bg(if self.render_active_styles.pocket {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
                                    .text_size(px(11.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(Self::apply_pocket_style),
                                    )
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
                                    .bg(if self.render_active_styles.hat {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
                                    .text_size(px(11.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(Self::apply_hat_style),
                                    )
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
                                    .bg(if self.render_active_styles.block {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
                                    .text_size(px(11.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(Self::apply_block_style),
                                    )
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
                                    .bg(if self.render_active_styles.tag {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
                                    .text_size(px(11.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(Self::apply_tag_style),
                                    )
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
                                    .bg(if self.render_active_styles.cite {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
                                    .text_size(px(11.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(Self::apply_cite_style),
                                    )
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
                                    .bg(if self.render_active_styles.highlight {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
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
                                    .bg(if self.render_active_styles.emphasis {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
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
                                    .bg(if self.render_active_styles.underline {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
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
                                    .bg(if self.render_active_styles.shrunk {
                                        colors.background
                                    } else {
                                        colors.surface_elevated
                                    })
                                    .text_size(px(11.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(colors.background))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(Self::apply_shrunk_style),
                                    )
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
                    ),
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

    fn render_editable_db8_document(
        &self,
        document: &OpenedDocument,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("db8-render-editor")
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_2()
            .track_focus(&self.render_editor_focus)
            .font_family(self.style_config.body.font_family.clone())
            .text_size(px(self.style_config.body.font_size_pt * 1.33))
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
            .min_h(px(24.0))
            .flex()
            .items_center()
            .gap_1()
            .when(semantic_block_class(block) == "pocket", |this| {
                this.justify_center()
                    .border_1()
                    .border_color(colors.text)
                    .px_2()
                    .py_1()
            })
            .when(semantic_block_class(block) == "hat", |this| {
                this.justify_center()
                    .underline()
                    .border_b_1()
                    .border_color(colors.text)
            })
            .when(semantic_block_class(block) == "block", |this| {
                this.justify_center()
                    .underline()
                    .text_size(px(16.0 * 1.33))
                    .font_weight(gpui::FontWeight::BOLD)
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
                    .gap_1()
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

fn rendered_line_text(block: &Db8Block) -> String {
    block
        .spans
        .iter()
        .map(|span| span.text.as_str())
        .collect::<Vec<_>>()
        .join("")
}

fn rendered_line_start_offset(document: &Db8Document, target_line: usize) -> usize {
    let mut offset = 0;
    for (line_index, block) in document.blocks.iter().enumerate() {
        if line_index == target_line {
            return offset;
        }
        offset += rendered_line_text(block).len() + 1;
    }
    offset
}

fn rendered_cursor_to_global_offset(document: &Db8Document, cursor: RawCursor) -> usize {
    let line_start = rendered_line_start_offset(document, cursor.line);
    let line_len = document
        .blocks
        .get(cursor.line)
        .map(rendered_line_text)
        .map_or(0, |line| line.len());
    line_start + cursor.column.min(line_len)
}

fn rendered_global_offset_to_cursor(document: &Db8Document, offset: usize) -> RawCursor {
    let mut line_start = 0usize;
    for (line_index, block) in document.blocks.iter().enumerate() {
        let line = rendered_line_text(block);
        let line_end = line_start + line.len();
        if offset <= line_end {
            return RawCursor {
                line: line_index,
                column: offset.saturating_sub(line_start),
            };
        }
        line_start = line_end + 1;
    }

    let last_line = document.blocks.len().saturating_sub(1);
    let last_column = document
        .blocks
        .get(last_line)
        .map(rendered_line_text)
        .map_or(0, |line| line.len());

    RawCursor {
        line: last_line,
        column: last_column,
    }
}

fn rendered_global_offset_to_raw_offset(raw: &str, rendered_offset: usize) -> usize {
    let parsed = Db8Document::parse(raw);
    let cursor = rendered_global_offset_to_cursor(&parsed, rendered_offset);
    rendered_cursor_to_raw_offset(raw, cursor)
}

fn rendered_cursor_to_raw_offset(raw: &str, cursor: RawCursor) -> usize {
    let mut raw_offset = 0usize;

    for (line_index, line) in raw.split('\n').enumerate() {
        if line_index == cursor.line {
            return raw_offset + rendered_line_to_raw_offset(line, cursor.column);
        }
        raw_offset += line.len() + 1;
    }

    raw.len()
}

fn raw_offset_to_rendered_cursor(raw: &str, raw_offset: usize) -> RawCursor {
    let parsed = Db8Document::parse(raw);
    let mut raw_line_start = 0usize;

    for (line_index, line) in raw.split('\n').enumerate() {
        let raw_line_end = raw_line_start + line.len();
        if raw_offset <= raw_line_end {
            let raw_column = raw_offset.saturating_sub(raw_line_start);
            return RawCursor {
                line: line_index,
                column: raw_line_offset_to_rendered_offset(line, raw_column),
            };
        }
        raw_line_start = raw_line_end + 1;
    }

    rendered_global_offset_to_cursor(&parsed, usize::MAX)
}

fn rendered_line_to_raw_offset(raw_line: &str, rendered_column: usize) -> usize {
    let mut i = 0usize;
    let mut rendered = 0usize;
    let bytes = raw_line.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'['
            && let Some(close_rel) = raw_line[i + 1..].find(']')
        {
            let close = i + 1 + close_rel;
            let candidate = &raw_line[i + 1..close];
            if let Some((styles, text_part)) = candidate.split_once(':') {
                let trimmed = text_part.trim_start_matches(' ');
                let dropped = text_part.len().saturating_sub(trimmed.len());
                let raw_text_start = i + 1 + styles.len() + 1 + dropped;
                let display_len = trimmed.len();

                if rendered_column <= rendered + display_len {
                    return raw_text_start
                        + rendered_column.saturating_sub(rendered).min(display_len);
                }

                rendered += display_len;
                i = close + 1;
                continue;
            }
        }

        if rendered == rendered_column {
            return i;
        }

        i += 1;
        rendered += 1;
    }

    raw_line.len()
}

fn raw_line_offset_to_rendered_offset(raw_line: &str, raw_column: usize) -> usize {
    let target = raw_column.min(raw_line.len());
    let mut i = 0usize;
    let mut rendered = 0usize;
    let bytes = raw_line.as_bytes();

    while i < target {
        if bytes[i] == b'['
            && let Some(close_rel) = raw_line[i + 1..].find(']')
        {
            let close = i + 1 + close_rel;
            let candidate = &raw_line[i + 1..close];
            if let Some((_styles, text_part)) = candidate.split_once(':') {
                let trimmed = text_part.trim_start_matches(' ');
                let raw_text_start = close.saturating_sub(trimmed.len());
                if target <= raw_text_start {
                    return rendered;
                }
                let local = target.saturating_sub(raw_text_start).min(trimmed.len());
                return rendered + local;
            }
        }
        i += 1;
        rendered += 1;
    }

    rendered
}

fn render_line_mouse_column(mouse_x: f32, sidebar_open: bool) -> usize {
    const RAW_EDITOR_PADDING_X: f32 = 16.0;
    const SIDEBAR_WIDTH: f32 = 240.0;

    let document_left = if sidebar_open { SIDEBAR_WIDTH } else { 0.0 };
    let text_left = document_left + RAW_EDITOR_PADDING_X;
    ((mouse_x - text_left) / RAW_EDITOR_CHAR_WIDTH)
        .round()
        .max(0.0) as usize
}

fn semantic_block_class(block: &Db8Block) -> &'static str {
    let Some(first) = block.spans.first() else {
        return "normal";
    };
    let styles = first.styles;

    if styles.pocket {
        "pocket"
    } else if styles.hat {
        "hat"
    } else if styles.block {
        "block"
    } else if styles.tag {
        "tag"
    } else if styles.cite {
        "cite"
    } else {
        "normal"
    }
}

fn render_db8_line_segments(
    block: &Db8Block,
    cursor_column: usize,
    show_cursor: bool,
    selected_range: Option<(usize, usize)>,
    style_config: &Db8StyleConfig,
    colors: AppColors,
) -> Vec<AnyElement> {
    let mut segments = Vec::new();
    let mut consumed = 0usize;
    let mut cursor_inserted = false;

    for span in &block.spans {
        let span_len = span.text.len();
        let span_start = consumed;
        let span_end = consumed + span_len;

        if show_cursor
            && !cursor_inserted
            && cursor_column >= span_start
            && cursor_column <= span_end
        {
            let local_cursor = cursor_column.saturating_sub(span_start).min(span_len);
            let (left, right) = span.text.split_at(local_cursor);
            if !left.is_empty() {
                segments.push(render_db8_text_segment(
                    left,
                    span.styles,
                    false,
                    style_config,
                    colors,
                ));
            }
            segments.push(raw_cursor_element(colors).into_any_element());
            cursor_inserted = true;
            if !right.is_empty() {
                segments.push(render_db8_text_segment(
                    right,
                    span.styles,
                    false,
                    style_config,
                    colors,
                ));
            }
        } else if let Some((sel_start, sel_end)) = selected_range {
            let local_start = sel_start.saturating_sub(span_start).min(span_len);
            let local_end = sel_end.saturating_sub(span_start).min(span_len);

            if sel_end <= span_start || sel_start >= span_end {
                if !span.text.is_empty() {
                    segments.push(render_db8_text_segment(
                        &span.text,
                        span.styles,
                        false,
                        style_config,
                        colors,
                    ));
                }
            } else {
                let (left, rest) = span.text.split_at(local_start);
                let selected_len = local_end.saturating_sub(local_start).min(rest.len());
                let (middle, right) = rest.split_at(selected_len);
                if !left.is_empty() {
                    segments.push(render_db8_text_segment(
                        left,
                        span.styles,
                        false,
                        style_config,
                        colors,
                    ));
                }
                if !middle.is_empty() {
                    segments.push(render_db8_text_segment(
                        middle,
                        span.styles,
                        true,
                        style_config,
                        colors,
                    ));
                }
                if !right.is_empty() {
                    segments.push(render_db8_text_segment(
                        right,
                        span.styles,
                        false,
                        style_config,
                        colors,
                    ));
                }
            }
        } else if !span.text.is_empty() {
            segments.push(render_db8_text_segment(
                &span.text,
                span.styles,
                false,
                style_config,
                colors,
            ));
        }

        consumed = span_end;
    }

    if show_cursor && !cursor_inserted && cursor_column == consumed {
        segments.push(raw_cursor_element(colors).into_any_element());
    }

    if segments.is_empty() {
        segments.push(div().child(" ").into_any_element());
    }

    segments
}

fn render_db8_text_segment(
    text: &str,
    styles: StyleSet,
    selected: bool,
    style_config: &Db8StyleConfig,
    colors: AppColors,
) -> AnyElement {
    let base_color = colors.text;
    let highlight_bg = color_from_hex(&style_config.highlight.background, colors.surface_elevated);
    let highlight_text = color_from_hex(&style_config.highlight.text_color, colors.text);
    let tag_color = colors.text;
    let cite_color = colors.text;

    div()
        .id(SharedString::from(format!(
            "db8-{}",
            semantic_span_class(styles)
        )))
        .font_family(style_config.body.font_family.clone())
        .text_size(px(if styles.shrunk {
            style_config.body.font_size_pt * style_config.shrunk.font_size_scale * 1.33
        } else if styles.pocket {
            style_config.pocket.font_size_pt * 1.33
        } else if styles.hat {
            style_config.hat.font_size_pt * 1.33
        } else if styles.block {
            16.0 * 1.33
        } else if styles.tag {
            style_config.tag.font_size_pt * 1.33
        } else if styles.cite {
            style_config.cite.font_size_pt * 1.33
        } else {
            style_config.normal.font_size_pt * 1.33
        }))
        .text_color(if styles.highlight {
            highlight_text
        } else if styles.tag {
            tag_color
        } else if styles.cite {
            cite_color
        } else {
            base_color
        })
        .when(
            styles.emphasis || styles.tag || styles.pocket || styles.hat || styles.block,
            |this| this.font_weight(gpui::FontWeight::BOLD),
        )
        .when(styles.underline, |this| this.underline())
        .when(styles.cite && style_config.cite.italic, |this| {
            this.italic()
        })
        .when(styles.highlight, |this| this.bg(highlight_bg))
        .when(styles.emphasis, |this| {
            this.border_1().border_color(colors.text).px_1()
        })
        .when(selected, |this| this.bg(colors.surface_elevated))
        .child(text.to_string())
        .into_any_element()
}

fn semantic_span_class(styles: StyleSet) -> &'static str {
    if styles.pocket {
        "pocket"
    } else if styles.hat {
        "hat"
    } else if styles.block {
        "block"
    } else if styles.tag {
        "tag"
    } else if styles.cite {
        "cite"
    } else if styles.highlight {
        "highlight"
    } else if styles.emphasis {
        "emphasis"
    } else if styles.underline {
        "underline"
    } else if styles.shrunk {
        "shrunk"
    } else {
        "normal"
    }
}

fn color_from_hex(hex: &str, fallback: gpui::Rgba) -> gpui::Rgba {
    let trimmed = hex.trim();
    let value = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if value.len() != 6 {
        return fallback;
    }

    u32::from_str_radix(value, 16)
        .ok()
        .map(gpui::rgb)
        .unwrap_or(fallback)
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
