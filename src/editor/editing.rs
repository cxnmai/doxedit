use super::*;

impl DebateEditor {
    pub(super) fn handle_editor_keystroke(
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
                    } else if keystroke.modifiers.shift {
                        self.render_paste_plain_clipboard(cx)
                    } else {
                        self.render_paste_clipboard(cx)
                    };
                    if changed {
                        self.autosave_raw_document();
                    }
                }
                "b" => {
                    if editing_render {
                        self.toggle_active_style("emphasis");
                    }
                }
                "u" => {
                    if editing_render {
                        self.toggle_active_style("underline");
                    }
                }
                "h" => {
                    if editing_render {
                        self.toggle_active_style("highlight");
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

    pub(super) fn raw_lines(&self) -> Vec<String> {
        self.opened_document
            .as_ref()
            .map(|document| split_raw_lines(&document.raw))
            .unwrap_or_default()
    }

    pub(super) fn update_raw_lines(&mut self, lines: Vec<String>) {
        if let Some(document) = self.opened_document.as_mut() {
            document.raw = lines.join("\n");
            document.parsed = Db8Document::parse(&document.raw);
        }
    }

    pub(super) fn raw_insert_text(&mut self, text: &str) -> bool {
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

    pub(super) fn raw_select_all(&mut self) {
        if let Some(document) = self.opened_document.as_ref() {
            let end = document.raw.len();
            self.raw_selection = Some(RawSelection {
                anchor: 0,
                head: end,
            });
            self.raw_cursor = raw_offset_to_cursor(&document.raw, end);
        }
    }

    pub(super) fn raw_copy_selection_or_line(&self, cx: &mut Context<Self>) {
        if let Some(selected_text) = self.raw_selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
        } else if let Some(line) = self.raw_lines().get(self.raw_cursor.line) {
            cx.write_to_clipboard(ClipboardItem::new_string(line.clone()));
        }
    }

    pub(super) fn raw_paste_clipboard(&mut self, cx: &mut Context<Self>) -> bool {
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

    pub(super) fn raw_insert_newline(&mut self) -> bool {
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

    pub(super) fn raw_backspace(&mut self) -> bool {
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

    pub(super) fn raw_delete(&mut self) -> bool {
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

    pub(super) fn raw_move_left(&mut self, selecting: bool) {
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

    pub(super) fn raw_move_right(&mut self, selecting: bool) {
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

    pub(super) fn raw_move_up(&mut self, selecting: bool) {
        let old_offset = self.raw_cursor_offset();
        if self.raw_cursor.line > 0 {
            self.raw_cursor.line -= 1;
            self.clamp_raw_cursor_column();
        }
        self.update_raw_selection_after_move(old_offset, selecting);
    }

    pub(super) fn raw_move_down(&mut self, selecting: bool) {
        let old_offset = self.raw_cursor_offset();
        let lines = self.raw_lines();
        if self.raw_cursor.line + 1 < lines.len() {
            self.raw_cursor.line += 1;
            self.clamp_raw_cursor_column();
        }
        self.update_raw_selection_after_move(old_offset, selecting);
    }

    pub(super) fn clamp_raw_cursor_column(&mut self) {
        let line_len = self
            .raw_lines()
            .get(self.raw_cursor.line)
            .map_or(0, String::len);
        self.raw_cursor.column = self.raw_cursor.column.min(line_len);
    }

    pub(super) fn autosave_raw_document(&self) {
        if let Some(document) = self.opened_document.as_ref() {
            let _ = std::fs::write(&document.path, document.parsed.to_bytes());
        }
    }

    pub(super) fn raw_cursor_offset(&self) -> usize {
        self.opened_document.as_ref().map_or(0, |document| {
            raw_cursor_to_offset(&document.raw, self.raw_cursor)
        })
    }

    pub(super) fn update_raw_selection_after_move(&mut self, old_offset: usize, selecting: bool) {
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

    pub(super) fn raw_selected_text(&self) -> Option<String> {
        let document = self.opened_document.as_ref()?;
        let selection = self.raw_selection?;
        let (start, end) = selection.ordered();

        if start == end {
            None
        } else {
            Some(document.raw[start..end].to_string())
        }
    }

    pub(super) fn raw_delete_selection(&mut self) -> bool {
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

    pub(super) fn ensure_render_cursor_in_bounds(&mut self) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };
        if document.parsed.blocks.is_empty() {
            self.render_cursor = RawCursor::default();
            return;
        }

        self.render_cursor.line = self
            .render_cursor
            .line
            .min(document.parsed.blocks.len().saturating_sub(1));
        let line_len = document
            .parsed
            .blocks
            .get(self.render_cursor.line)
            .map(rendered_line_text)
            .map_or(0, |line| line.len());
        self.render_cursor.column = self.render_cursor.column.min(line_len);
    }

    pub(super) fn render_select_all(&mut self) {
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

    pub(super) fn render_copy_selection_or_line(&mut self, cx: &mut Context<Self>) {
        if let Some(fragment) = self.render_selected_fragment() {
            let text = rendered_document_text(&fragment);
            self.render_clipboard = Some(fragment);
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            return;
        }

        if let Some(document) = self.opened_document.as_ref()
            && let Some(block) = document.parsed.blocks.get(self.render_cursor.line)
        {
            let fragment = Db8Document {
                version: 1,
                blocks: vec![block.clone()],
            };
            self.render_clipboard = Some(fragment);
            cx.write_to_clipboard(ClipboardItem::new_string(rendered_line_text(block)));
        }
    }

    pub(super) fn render_paste_clipboard(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(fragment) = self.render_clipboard.clone() {
            return self.render_insert_fragment(&fragment);
        }

        self.render_paste_plain_clipboard(cx)
    }

    pub(super) fn render_paste_plain_clipboard(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return false;
        };
        self.render_insert_text(&text)
    }

    pub(super) fn render_selected_fragment(&self) -> Option<Db8Document> {
        let document = self.opened_document.as_ref()?;
        let selection = self.render_selection?;
        let (start, end) = selection.ordered();
        if start == end {
            return None;
        }
        copy_rendered_range(&document.parsed, start, end)
    }

    pub(super) fn render_insert_fragment(&mut self, fragment: &Db8Document) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            if delete_rendered_range(&mut document.parsed, start, end) {
                self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, start);
            }
        }

        self.render_cursor =
            insert_rendered_fragment(&mut document.parsed, self.render_cursor, fragment);
        normalize_document_spans(&mut document.parsed);
        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        true
    }

    pub(super) fn render_insert_text(&mut self, text: &str) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            if delete_rendered_range(&mut document.parsed, start, end) {
                self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, start);
            }
        }

        let insertion_styles = if self.render_active_styles.is_plain() {
            rendered_cursor_styles(&document.parsed, self.render_cursor).unwrap_or_default()
        } else {
            self.render_active_styles
        };
        self.render_cursor = insert_rendered_text(
            &mut document.parsed,
            self.render_cursor,
            text,
            insertion_styles,
        );
        normalize_document_spans(&mut document.parsed);
        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        self.ensure_render_cursor_in_bounds();
        true
    }

    pub(super) fn render_backspace(&mut self) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            if delete_rendered_range(&mut document.parsed, start, end) {
                document.raw = document.parsed.to_db8();
                document.parsed = Db8Document::parse(&document.raw);
                self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, start);
                self.ensure_render_cursor_in_bounds();
                return true;
            }
            return false;
        }

        let cursor_offset = rendered_cursor_to_global_offset(&document.parsed, self.render_cursor);
        if cursor_offset == 0 {
            return false;
        }

        let previous = previous_rendered_global_offset(&document.parsed, cursor_offset);
        if !delete_rendered_range(&mut document.parsed, previous, cursor_offset) {
            return false;
        }
        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, previous);
        self.ensure_render_cursor_in_bounds();
        true
    }

    pub(super) fn render_delete(&mut self) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            if delete_rendered_range(&mut document.parsed, start, end) {
                document.raw = document.parsed.to_db8();
                document.parsed = Db8Document::parse(&document.raw);
                self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, start);
                self.ensure_render_cursor_in_bounds();
                return true;
            }
            return false;
        }

        let cursor_offset = rendered_cursor_to_global_offset(&document.parsed, self.render_cursor);
        let next = next_rendered_global_offset(&document.parsed, cursor_offset);
        if next <= cursor_offset {
            return false;
        }

        if !delete_rendered_range(&mut document.parsed, cursor_offset, next) {
            return false;
        }
        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, cursor_offset);
        self.ensure_render_cursor_in_bounds();
        true
    }

    pub(super) fn render_insert_newline(&mut self) -> bool {
        let Some(document) = self.opened_document.as_mut() else {
            return false;
        };

        let line_index = self
            .render_cursor
            .line
            .min(document.parsed.blocks.len().saturating_sub(1));

        if let Some(selection) = self.render_selection.take() {
            let (start, end) = selection.ordered();
            if delete_rendered_range(&mut document.parsed, start, end) {
                document.raw = document.parsed.to_db8();
                document.parsed = Db8Document::parse(&document.raw);
                self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, start);
            }
        }

        let Some(current_block) = document.parsed.blocks.get(line_index).cloned() else {
            return false;
        };

        let target_column = self
            .render_cursor
            .column
            .min(rendered_line_text(&current_block).len());
        let inherited_styles = current_block
            .spans
            .first()
            .map(|span| span.styles)
            .unwrap_or_default();
        let mut left_spans: Vec<Db8Span> = Vec::new();
        let mut right_spans: Vec<Db8Span> = Vec::new();
        let mut consumed = 0usize;

        for span in current_block.spans {
            let span_len = span.text.len();
            let span_start = consumed;
            let span_end = consumed + span_len;

            if target_column <= span_start {
                let mut span = span;
                clear_primary_semantic_styles(&mut span.styles);
                right_spans.push(span);
            } else if target_column >= span_end {
                let mut span = span;
                clear_primary_semantic_styles(&mut span.styles);
                left_spans.push(span);
            } else {
                let local = target_column.saturating_sub(span_start).min(span_len);
                let (left_text, right_text) = span.text.split_at(local);
                let mut styles = span.styles;
                clear_primary_semantic_styles(&mut styles);
                if !left_text.is_empty() {
                    left_spans.push(Db8Span {
                        text: left_text.to_string(),
                        styles,
                    });
                }
                if !right_text.is_empty() {
                    right_spans.push(Db8Span {
                        text: right_text.to_string(),
                        styles,
                    });
                }
            }

            consumed = span_end;
        }

        if left_spans.is_empty() {
            left_spans.push(Db8Span {
                text: String::new(),
                styles: inherited_styles,
            });
        }
        if right_spans.is_empty() {
            right_spans.push(Db8Span {
                text: String::new(),
                styles: StyleSet::default(),
            });
        }

        if let Some(block) = document.parsed.blocks.get_mut(line_index) {
            block.spans = left_spans;
        }
        document.parsed.blocks.insert(
            line_index + 1,
            Db8Block {
                source_line: line_index + 1,
                style: BlockStyle::Normal,
                spans: right_spans,
            },
        );

        for (idx, block) in document.parsed.blocks.iter_mut().enumerate() {
            block.source_line = idx;
        }

        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        self.render_cursor = RawCursor {
            line: line_index + 1,
            column: 0,
        };
        self.render_selection = None;
        self.ensure_render_cursor_in_bounds();
        true
    }

    pub(super) fn render_move_left(&mut self, selecting: bool) {
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

    pub(super) fn render_move_right(&mut self, selecting: bool) {
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

    pub(super) fn render_move_up(&mut self, selecting: bool) {
        let old_offset = self.render_cursor_offset();
        if self.render_cursor.line > 0 {
            self.render_cursor.line -= 1;
            self.clamp_render_cursor_column();
        }
        self.update_render_selection_after_move(old_offset, selecting);
    }

    pub(super) fn render_move_down(&mut self, selecting: bool) {
        let old_offset = self.render_cursor_offset();
        if let Some(document) = self.opened_document.as_ref()
            && self.render_cursor.line + 1 < document.parsed.blocks.len()
        {
            self.render_cursor.line += 1;
            self.clamp_render_cursor_column();
        }
        self.update_render_selection_after_move(old_offset, selecting);
    }

    pub(super) fn clamp_render_cursor_column(&mut self) {
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

    pub(super) fn render_cursor_offset(&self) -> usize {
        self.opened_document.as_ref().map_or(0, |document| {
            rendered_cursor_to_global_offset(&document.parsed, self.render_cursor)
        })
    }

    pub(super) fn update_render_selection_after_move(
        &mut self,
        old_offset: usize,
        selecting: bool,
    ) {
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

    pub(super) fn start_render_end_mouse_selection(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };
        let end = rendered_document_len(&document.parsed);
        self.render_mouse_selecting = false;
        self.render_mouse_anchor = Some(end);
        self.render_selection = None;
        self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, end);
        window.focus(&self.render_editor_focus);
        cx.notify();
    }

    pub(super) fn update_render_end_mouse_selection(
        &mut self,
        _event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(anchor) = self.render_mouse_anchor else {
            return;
        };
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };
        let end = rendered_document_len(&document.parsed);
        if anchor == end {
            return;
        }
        self.render_mouse_selecting = true;
        self.render_selection = Some(RawSelection { anchor, head: end });
        self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, end);
        cx.notify();
    }

    pub(super) fn start_render_mouse_selection(
        &mut self,
        line: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let Some(block) = document.parsed.blocks.get(line) else {
            return;
        };
        let line_text = rendered_line_text(block);
        let line_char_count = line_text.chars().count();
        let column = render_line_mouse_column(
            block,
            f32::from(event.position.x),
            self.sidebar_open,
            f32::from(window.viewport_size().width),
            &self.style_config,
        )
        .min(line_char_count);

        let line_start = rendered_line_start_offset(&document.parsed, line);
        let line_len = line_text.len();

        self.render_mouse_selecting = false;
        window.focus(&self.render_editor_focus);

        if event.click_count >= 3 {
            self.render_mouse_anchor = None;
            self.render_selection = Some(RawSelection {
                anchor: line_start,
                head: line_start + line_len,
            });
            self.render_cursor = RawCursor {
                line,
                column: line_len,
            };
            cx.notify();
            return;
        }

        if event.click_count == 2 {
            let (word_start, word_end) = word_range_at_column(&line_text, column);
            self.render_mouse_anchor = None;
            self.render_selection = Some(RawSelection {
                anchor: line_start + word_start,
                head: line_start + word_end,
            });
            self.render_cursor = RawCursor {
                line,
                column: word_end,
            };
            cx.notify();
            return;
        }

        self.render_mouse_anchor = Some(rendered_cursor_to_global_offset(
            &document.parsed,
            RawCursor { line, column },
        ));
        self.render_selection = None;
        self.render_cursor = RawCursor { line, column };
        cx.notify();
    }

    pub(super) fn update_render_mouse_selection(
        &mut self,
        line: usize,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(anchor) = self.render_mouse_anchor else {
            return;
        };
        let Some(document) = self.opened_document.as_ref() else {
            return;
        };

        let Some(block) = document.parsed.blocks.get(line) else {
            return;
        };
        let line_text = rendered_line_text(block);
        let line_char_count = line_text.chars().count();
        let column = render_line_mouse_column(
            block,
            f32::from(event.position.x),
            self.sidebar_open,
            f32::from(window.viewport_size().width),
            &self.style_config,
        )
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

    pub(super) fn finish_render_mouse_selection(
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

    pub(super) fn cursor_has_style(&self, style_token: &str) -> bool {
        let Some(document) = self.opened_document.as_ref() else {
            return false;
        };
        if document.parsed.blocks.is_empty() {
            return false;
        }

        let line_index = self
            .render_cursor
            .line
            .min(document.parsed.blocks.len().saturating_sub(1));
        let Some(block) = document.parsed.blocks.get(line_index) else {
            return false;
        };

        match style_token {
            "normal" => block.style == BlockStyle::Normal,
            "pocket" => block.style == BlockStyle::Pocket,
            "hat" => block.style == BlockStyle::Hat,
            "block" => block.style == BlockStyle::Block,
            "tag" => block.style == BlockStyle::Tag,
            "cite" => block.style == BlockStyle::Cite,
            "highlight" => rendered_cursor_styles(&document.parsed, self.render_cursor)
                .is_some_and(|styles| styles.highlight),
            "emphasis" => rendered_cursor_styles(&document.parsed, self.render_cursor)
                .is_some_and(|styles| styles.emphasis),
            "underline" => rendered_cursor_styles(&document.parsed, self.render_cursor)
                .is_some_and(|styles| styles.underline),
            "shrunk" => rendered_cursor_styles(&document.parsed, self.render_cursor)
                .is_some_and(|styles| styles.shrunk),
            _ => false,
        }
    }

    pub(super) fn apply_semantic_style_to_selection(
        &mut self,
        style_token: &str,
        cx: &mut Context<Self>,
    ) {
        self.ensure_render_cursor_in_bounds();

        if is_line_semantic_style(style_token) {
            if let Some(selection) = self.render_selection {
                let (start, end) = selection.ordered();
                self.apply_line_semantic_style_to_range(style_token, start, end);
                self.autosave_raw_document();
                cx.notify();
                return;
            }

            self.apply_line_semantic_style(style_token);
            self.autosave_raw_document();
            cx.notify();
            return;
        }

        if let Some(selection) = self.render_selection {
            let (start, end) = selection.ordered();
            if start == end {
                self.render_selection = None;
                self.toggle_active_style(style_token);
                self.autosave_raw_document();
                cx.notify();
                return;
            }

            let Some(document) = self.opened_document.as_mut() else {
                return;
            };

            let remove_style =
                rendered_range_has_inline_style(&document.parsed, start, end, style_token);
            if apply_inline_style_to_rendered_range(
                &mut document.parsed,
                start,
                end,
                style_token,
                !remove_style,
            ) {
                document.raw = document.parsed.to_db8();
                document.parsed = Db8Document::parse(&document.raw);
                let bounded_end = rendered_document_len(&document.parsed).min(end);
                self.render_selection = Some(RawSelection {
                    anchor: start,
                    head: bounded_end,
                });
                self.render_cursor =
                    rendered_global_offset_to_cursor(&document.parsed, bounded_end);
                self.autosave_raw_document();
                cx.notify();
            }
            return;
        }

        self.toggle_active_style(style_token);
        self.autosave_raw_document();
        cx.notify();
    }

    pub(super) fn apply_line_semantic_style_to_range(
        &mut self,
        style_token: &str,
        start: usize,
        end: usize,
    ) {
        let Some(document) = self.opened_document.as_mut() else {
            return;
        };
        if document.parsed.blocks.is_empty() || start == end {
            return;
        }

        let start_cursor = rendered_global_offset_to_cursor(&document.parsed, start);
        let mut end_cursor = rendered_global_offset_to_cursor(&document.parsed, end);
        if end_cursor.column == 0 && end_cursor.line > start_cursor.line {
            end_cursor.line = end_cursor.line.saturating_sub(1);
            end_cursor.column = document
                .parsed
                .blocks
                .get(end_cursor.line)
                .map(rendered_line_text)
                .map_or(0, |text| text.len());
        }

        let start_line = start_cursor
            .line
            .min(document.parsed.blocks.len().saturating_sub(1));
        let end_line = end_cursor
            .line
            .min(document.parsed.blocks.len().saturating_sub(1));
        let target_style = block_style_from_token(style_token);
        let all_already_style = (start_line..=end_line).all(|line| {
            document
                .parsed
                .blocks
                .get(line)
                .is_some_and(|block| block.style == target_style)
        });
        let new_style = if all_already_style {
            BlockStyle::Normal
        } else {
            target_style
        };

        for line in start_line..=end_line {
            if let Some(block) = document.parsed.blocks.get_mut(line) {
                block.style = new_style;
                for span in &mut block.spans {
                    clear_primary_semantic_styles(&mut span.styles);
                }
            }
        }

        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        let bounded_end = rendered_document_len(&document.parsed).min(end);
        self.render_selection = Some(RawSelection {
            anchor: start,
            head: bounded_end,
        });
        self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, bounded_end);
    }

    pub(super) fn apply_line_semantic_style(&mut self, style_token: &str) {
        let Some(document) = self.opened_document.as_mut() else {
            return;
        };
        if document.parsed.blocks.is_empty() {
            return;
        }

        let target_line = self
            .render_cursor
            .line
            .min(document.parsed.blocks.len().saturating_sub(1));
        let cursor_offset = rendered_cursor_to_global_offset(
            &document.parsed,
            RawCursor {
                line: target_line,
                column: self.render_cursor.column,
            },
        );
        let Some(block) = document.parsed.blocks.get_mut(target_line) else {
            return;
        };

        let mut has_visible_content = false;
        let target_style = block_style_from_token(style_token);
        block.style = if block.style == target_style {
            BlockStyle::Normal
        } else {
            target_style
        };
        for span in &mut block.spans {
            if !span.text.trim().is_empty() {
                has_visible_content = true;
            }
            clear_primary_semantic_styles(&mut span.styles);
        }

        if !has_visible_content {
            block.spans = vec![crate::db8_document::Db8Span {
                text: String::new(),
                styles: StyleSet::default(),
            }];
        }

        document.raw = document.parsed.to_db8();
        document.parsed = Db8Document::parse(&document.raw);
        self.render_cursor = rendered_global_offset_to_cursor(&document.parsed, cursor_offset);
        self.render_selection = None;
    }

    pub(super) fn toggle_active_style(&mut self, style_token: &str) {
        match style_token {
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

    pub(super) fn style_mutation(
        &mut self,
        mutator: impl FnOnce(&mut Db8StyleConfig),
        cx: &mut Context<Self>,
    ) {
        mutator(&mut self.style_config);
        self.generated_css = style_config_to_css(&self.style_config);
        let _ = save_style_config(&self.style_config_path, &self.style_config);
        cx.notify();
    }

    pub(super) fn apply_pocket_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("pocket", cx);
    }

    pub(super) fn apply_hat_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("hat", cx);
    }

    pub(super) fn apply_block_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("block", cx);
    }

    pub(super) fn apply_tag_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("tag", cx);
    }

    pub(super) fn apply_cite_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("cite", cx);
    }

    pub(super) fn apply_normal_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.render_active_styles = StyleSet::default();
        self.render_selection = None;
        if let Some(document) = self.opened_document.as_mut()
            && let Some(block) = document.parsed.blocks.get_mut(self.render_cursor.line)
        {
            block.style = BlockStyle::Normal;
            document.raw = document.parsed.to_db8();
            document.parsed = Db8Document::parse(&document.raw);
            self.autosave_raw_document();
        }
        cx.notify();
    }

    pub(super) fn apply_highlight_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("highlight", cx);
    }

    pub(super) fn apply_emphasis_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("emphasis", cx);
    }

    pub(super) fn apply_underline_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("underline", cx);
    }

    pub(super) fn apply_shrunk_style(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.render_editor_focus);
        self.apply_semantic_style_to_selection("shrunk", cx);
    }
    pub(super) fn start_raw_document_mouse_selection(
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

    pub(super) fn update_raw_document_mouse_selection(
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

    pub(super) fn start_raw_mouse_selection(
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

    pub(super) fn update_raw_mouse_selection(
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

    pub(super) fn finish_raw_mouse_selection(
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
}
