use super::*;

impl DebateEditor {
    pub(super) fn render_project_tree(
        &self,
        colors: AppColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                                    .id("new-root-directory")
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
                                    .on_click(cx.listener(Self::create_root_directory))
                                    .child("+ Dir"),
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
                let path_for_new_file = path.clone();
                let path_for_new_dir = path.clone();
                let path_for_delete = path.clone();
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
                            .child(div().flex_1().child(name.clone()))
                            .child(
                                div()
                                    .h(px(20.0))
                                    .px_1()
                                    .rounded_sm()
                                    .text_color(colors.text)
                                    .hover(|this| this.bg(colors.background))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.create_db8_file_in_directory(
                                                path_for_new_file.clone(),
                                                cx,
                                            );
                                        }),
                                    )
                                    .child("+F"),
                            )
                            .child(
                                div()
                                    .h(px(20.0))
                                    .px_1()
                                    .rounded_sm()
                                    .text_color(colors.text)
                                    .hover(|this| this.bg(colors.background))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.create_directory_in_directory(
                                                path_for_new_dir.clone(),
                                                cx,
                                            );
                                        }),
                                    )
                                    .child("+D"),
                            )
                            .child(
                                div()
                                    .h(px(20.0))
                                    .px_1()
                                    .rounded_sm()
                                    .text_color(colors.text_muted)
                                    .hover(|this| this.bg(colors.background))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.delete_tree_path(path_for_delete.clone(), cx);
                                        }),
                                    )
                                    .child("×"),
                            ),
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
                let path_for_delete = path.clone();
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
                    .child(div().flex_1().child(name.clone()))
                    .child(
                        div()
                            .h(px(20.0))
                            .px_1()
                            .rounded_sm()
                            .text_color(colors.text_muted)
                            .hover(|this| this.bg(colors.background))
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.delete_tree_path(path_for_delete.clone(), cx);
                                }),
                            )
                            .child("×"),
                    )
                    .into_any_element()
            }
        }
    }
    pub(super) fn toggle_folder(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if !self.expanded_folders.remove(&path) {
            self.expanded_folders.insert(path);
        }

        cx.notify();
    }

    pub(super) fn refresh_project_tree(&mut self, cx: &mut Context<Self>) {
        self.project_tree = ProjectTree::load(self.project_tree.root.clone());
        let folder_paths = self.project_tree.folder_paths();
        self.expanded_folders
            .retain(|path| folder_paths.contains(path));
        self.expanded_folders.extend(folder_paths);
        cx.notify();
    }

    pub(super) fn create_root_db8_file(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.create_db8_file_in_directory(self.project_tree.root.clone(), cx);
    }

    pub(super) fn create_root_directory(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.create_directory_in_directory(self.project_tree.root.clone(), cx);
    }

    pub(super) fn create_db8_file_in_directory(
        &mut self,
        directory: PathBuf,
        cx: &mut Context<Self>,
    ) {
        let file_path = next_available_path(&directory, "new_file", "db8");
        let document = Db8Document::empty();
        if std::fs::write(&file_path, document.to_bytes()).is_ok() {
            self.refresh_project_tree(cx);
            self.select_file(file_path, cx);
        }
    }

    pub(super) fn create_directory_in_directory(
        &mut self,
        directory: PathBuf,
        cx: &mut Context<Self>,
    ) {
        let folder_path = next_available_folder_path(&directory, "new_folder");
        if std::fs::create_dir_all(&folder_path).is_ok() {
            self.expanded_folders.insert(directory);
            self.expanded_folders.insert(folder_path);
            self.refresh_project_tree(cx);
        }
    }

    pub(super) fn delete_tree_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if path == self.project_tree.root {
            return;
        }

        let delete_result = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };

        if delete_result.is_err() {
            return;
        }

        if self
            .selected_file
            .as_ref()
            .is_some_and(|selected| selected == &path)
        {
            self.selected_file = None;
            self.opened_document = None;
        }

        if let Some(document) = self.opened_document.as_ref()
            && document.path.starts_with(&path)
        {
            self.selected_file = None;
            self.opened_document = None;
        }

        self.expanded_folders.remove(&path);
        self.refresh_project_tree(cx);
    }

    pub(super) fn can_drop_tree_file_to(
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

    pub(super) fn move_file_to_directory(
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
}
