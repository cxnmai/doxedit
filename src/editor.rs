use crate::{
    db8_document::{BlockStyle, Db8Block, Db8Document, Db8Span, StyleSet},
    db8_style::{Db8StyleConfig, load_or_create_style_config, save_style_config, style_config_to_css},
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
    time::Duration,
};

#[path = "editor/app_shell.rs"]
mod app_shell;
#[path = "editor/document_outline.rs"]
mod document_outline;
#[path = "editor/document_view.rs"]
mod document_view;
#[path = "editor/editing.rs"]
mod editing;
#[path = "editor/project_sidebar.rs"]
mod project_sidebar;

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
    expanded_document_outline: HashSet<String>,
    render_cursor: RawCursor,
    render_selection: Option<RawSelection>,
    render_mouse_selecting: bool,
    render_mouse_anchor: Option<usize>,
    render_active_styles: StyleSet,
    render_clipboard: Option<Db8Document>,
    style_config_path: PathBuf,
    style_config: Db8StyleConfig,
    generated_css: String,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DocumentViewMode {
    Raw,
    Render,
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
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let _ = this.update(cx, |this, cx| {
                    this.refresh_project_tree(cx);
                });
            }
        })
        .detach();
        let style_config_path = project_root.join(".db8_style.json");
        let style_config = load_or_create_style_config(&style_config_path);
        let generated_css = style_config_to_css(&style_config);

        Self {
            color_mode: ColorMode::initial(),
            document_view_mode: DocumentViewMode::Render,
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
            expanded_document_outline: HashSet::new(),
            render_cursor: RawCursor::default(),
            render_selection: None,
            render_mouse_selecting: false,
            render_mouse_anchor: None,
            render_active_styles: StyleSet::default(),
            render_clipboard: None,
            style_config_path,
            style_config,
            generated_css,
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
                    this.expanded_document_outline.clear();
                    this.render_selection = None;
                    this.render_mouse_selecting = false;
                    this.render_mouse_anchor = None;
                    this.render_active_styles = StyleSet::default();
                    this.render_clipboard = None;
                    this.style_config_path = style_config_path;
                    this.style_config = style_config;
                    this.generated_css = generated_css;
                    this.expanded_folders = expanded_folders;
                    this.sidebar_open = true;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn select_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let bytes = std::fs::read(&path).unwrap_or_default();
        let parsed = Db8Document::from_bytes(&bytes);
        let raw = parsed.to_db8();
        let expanded_document_outline =
            document_outline::default_document_outline_expansion(&parsed);

        self.selected_file = Some(path.clone());
        self.opened_document = Some(OpenedDocument { path, raw, parsed });
        self.raw_cursor = RawCursor::default();
        self.raw_selection = None;
        self.raw_mouse_selecting = false;
        self.raw_mouse_anchor = None;
        self.render_cursor = RawCursor::default();
        self.expanded_document_outline = expanded_document_outline;
        self.render_selection = None;
        self.render_mouse_selecting = false;
        self.render_mouse_anchor = None;
        self.render_active_styles = StyleSet::default();
        cx.notify();
    }
}

include!("editor/document_ops.rs");
