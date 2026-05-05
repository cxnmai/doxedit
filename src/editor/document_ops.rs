fn word_range_at_column(text: &str, column: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }

    let column = column.min(text.len());
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let char_index = chars
        .iter()
        .position(|(byte, _)| *byte >= column)
        .unwrap_or(chars.len().saturating_sub(1));
    let char_index = if char_index > 0 && column == text.len() {
        char_index.saturating_sub(1)
    } else {
        char_index
    };

    let is_word = |ch: char| ch.is_alphanumeric() || ch == '_';
    if !chars.get(char_index).is_some_and(|(_, ch)| is_word(*ch)) {
        let start = chars.get(char_index).map_or(text.len(), |(byte, _)| *byte);
        let end = chars
            .get(char_index + 1)
            .map_or(text.len(), |(byte, _)| *byte);
        return (start, end);
    }

    let mut start_index = char_index;
    while start_index > 0
        && chars
            .get(start_index - 1)
            .is_some_and(|(_, ch)| is_word(*ch))
    {
        start_index -= 1;
    }

    let mut end_index = char_index + 1;
    while end_index < chars.len() && chars.get(end_index).is_some_and(|(_, ch)| is_word(*ch)) {
        end_index += 1;
    }

    let start = chars[start_index].0;
    let end = chars.get(end_index).map_or(text.len(), |(byte, _)| *byte);
    (start, end)
}

fn next_available_path(directory: &Path, base_name: &str, extension: &str) -> PathBuf {
    let mut index = 0usize;
    loop {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("_{}", index)
        };
        let candidate = directory.join(format!("{}{}.{}", base_name, suffix, extension));
        if !candidate.exists() {
            return candidate;
        }
        index += 1;
    }
}

fn next_available_folder_path(directory: &Path, base_name: &str) -> PathBuf {
    let mut index = 0usize;
    loop {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("_{}", index)
        };
        let candidate = directory.join(format!("{}{}", base_name, suffix));
        if !candidate.exists() {
            return candidate;
        }
        index += 1;
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

fn rendered_cursor_styles(document: &Db8Document, cursor: RawCursor) -> Option<StyleSet> {
    let block = document.blocks.get(cursor.line)?;
    let mut consumed = 0usize;
    let mut previous = None;

    for span in &block.spans {
        let span_end = consumed + span.text.len();
        if cursor.column < span_end || (cursor.column == span_end && !span.text.is_empty()) {
            return Some(span.styles);
        }
        previous = Some(span.styles);
        consumed = span_end;
    }

    previous.or_else(|| block.spans.first().map(|span| span.styles))
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

fn rendered_document_len(document: &Db8Document) -> usize {
    document
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            rendered_line_text(block).len() + usize::from(index + 1 < document.blocks.len())
        })
        .sum()
}

fn previous_rendered_global_offset(document: &Db8Document, offset: usize) -> usize {
    if offset == 0 {
        0
    } else {
        previous_char_boundary(&rendered_document_text(document), offset)
    }
}

fn next_rendered_global_offset(document: &Db8Document, offset: usize) -> usize {
    let text = rendered_document_text(document);
    if offset >= text.len() {
        offset
    } else {
        next_char_boundary(&text, offset)
    }
}

fn rendered_document_text(document: &Db8Document) -> String {
    document
        .blocks
        .iter()
        .map(rendered_line_text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn copy_rendered_range(document: &Db8Document, start: usize, end: usize) -> Option<Db8Document> {
    let rendered_len = rendered_document_len(document);
    let start = start.min(rendered_len);
    let end = end.min(rendered_len);
    if start >= end || document.blocks.is_empty() {
        return None;
    }

    let start_cursor = rendered_global_offset_to_cursor(document, start);
    let end_cursor = rendered_global_offset_to_cursor(document, end);
    let start_line = start_cursor
        .line
        .min(document.blocks.len().saturating_sub(1));
    let end_line = end_cursor.line.min(document.blocks.len().saturating_sub(1));
    let mut blocks = Vec::new();

    for line in start_line..=end_line {
        let block = &document.blocks[line];
        let line_len = rendered_line_text(block).len();
        let local_start = if line == start_line {
            start_cursor.column
        } else {
            0
        };
        let local_end = if line == end_line {
            end_cursor.column
        } else {
            line_len
        };
        let (_, rest) = split_spans_at(&block.spans, local_start);
        let (spans, _) = split_spans_at(&rest, local_end.saturating_sub(local_start));
        blocks.push(Db8Block {
            source_line: blocks.len(),
            style: block.style,
            spans: normalize_spans(spans),
        });
    }

    Some(Db8Document { version: 1, blocks })
}

fn insert_rendered_fragment(
    document: &mut Db8Document,
    cursor: RawCursor,
    fragment: &Db8Document,
) -> RawCursor {
    if fragment.blocks.is_empty() {
        return cursor;
    }

    if document.blocks.is_empty() {
        *document = Db8Document::empty();
    }

    let line = cursor.line.min(document.blocks.len().saturating_sub(1));
    let column = cursor.column.min(
        document
            .blocks
            .get(line)
            .map(rendered_line_text)
            .map_or(0, |text| text.len()),
    );
    let (mut left, right) = split_spans_at(&document.blocks[line].spans, column);

    if fragment.blocks.len() == 1 {
        document.blocks[line].style = fragment.blocks[0].style;
        left.extend(fragment.blocks[0].spans.clone());
        left.extend(right);
        document.blocks[line].spans = normalize_spans(left);
        return RawCursor {
            line,
            column: column + rendered_line_text(&fragment.blocks[0]).len(),
        };
    }

    left.extend(fragment.blocks[0].spans.clone());
    document.blocks[line].spans = normalize_spans(left);

    let mut insert_at = line + 1;
    for block in fragment
        .blocks
        .iter()
        .skip(1)
        .take(fragment.blocks.len().saturating_sub(2))
    {
        let mut block = block.clone();
        block.source_line = insert_at;
        document.blocks.insert(insert_at, block);
        insert_at += 1;
    }

    let last = fragment.blocks.last().cloned().unwrap();
    let mut last_spans = last.spans;
    last_spans.extend(right);
    document.blocks.insert(
        insert_at,
        Db8Block {
            source_line: insert_at,
            style: last.style,
            spans: normalize_spans(last_spans),
        },
    );

    for (idx, block) in document.blocks.iter_mut().enumerate() {
        block.source_line = idx;
    }

    RawCursor {
        line: insert_at,
        column: rendered_line_text(&fragment.blocks.last().unwrap()).len(),
    }
}

fn insert_rendered_text(
    document: &mut Db8Document,
    cursor: RawCursor,
    text: &str,
    styles: StyleSet,
) -> RawCursor {
    if document.blocks.is_empty() {
        document.blocks.push(Db8Block {
            source_line: 0,
            style: BlockStyle::Normal,
            spans: vec![Db8Span {
                text: String::new(),
                styles: StyleSet::default(),
            }],
        });
    }

    let line = cursor.line.min(document.blocks.len().saturating_sub(1));
    let column = cursor.column.min(
        document
            .blocks
            .get(line)
            .map(rendered_line_text)
            .map_or(0, |text| text.len()),
    );
    let (mut left, right) = split_spans_at(&document.blocks[line].spans, column);
    let parts: Vec<&str> = text.split('\n').collect();

    if parts.len() == 1 {
        if !text.is_empty() {
            left.push(Db8Span {
                text: text.to_string(),
                styles,
            });
        }
        left.extend(right);
        document.blocks[line].spans = normalize_spans(left);
        return RawCursor {
            line,
            column: column + text.len(),
        };
    }

    if !parts[0].is_empty() {
        left.push(Db8Span {
            text: parts[0].to_string(),
            styles,
        });
    }
    document.blocks[line].spans = normalize_spans(left);

    let mut insert_at = line + 1;
    for middle in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
        document.blocks.insert(
            insert_at,
            Db8Block {
                source_line: insert_at,
                style: BlockStyle::Normal,
                spans: normalize_spans(vec![Db8Span {
                    text: (*middle).to_string(),
                    styles,
                }]),
            },
        );
        insert_at += 1;
    }

    let last_text = parts.last().copied().unwrap_or("");
    let mut last_spans = Vec::new();
    if !last_text.is_empty() {
        last_spans.push(Db8Span {
            text: last_text.to_string(),
            styles,
        });
    }
    last_spans.extend(right);
    document.blocks.insert(
        insert_at,
        Db8Block {
            source_line: insert_at,
            style: BlockStyle::Normal,
            spans: normalize_spans(last_spans),
        },
    );

    for (idx, block) in document.blocks.iter_mut().enumerate() {
        block.source_line = idx;
    }

    RawCursor {
        line: insert_at,
        column: last_text.len(),
    }
}

fn delete_rendered_range(document: &mut Db8Document, start: usize, end: usize) -> bool {
    let rendered_len = rendered_document_len(document);
    let start = start.min(rendered_len);
    let end = end.min(rendered_len);
    if start >= end || document.blocks.is_empty() {
        return false;
    }

    let start_cursor = rendered_global_offset_to_cursor(document, start);
    let end_cursor = rendered_global_offset_to_cursor(document, end);

    if start_cursor.line == end_cursor.line {
        if let Some(block) = document.blocks.get_mut(start_cursor.line) {
            block.spans = delete_from_spans(&block.spans, start_cursor.column, end_cursor.column);
        }
    } else {
        let start_line = start_cursor
            .line
            .min(document.blocks.len().saturating_sub(1));
        let end_line = end_cursor.line.min(document.blocks.len().saturating_sub(1));
        let (_, mut right_tail) =
            split_spans_at(&document.blocks[end_line].spans, end_cursor.column);
        let (mut left_head, _) =
            split_spans_at(&document.blocks[start_line].spans, start_cursor.column);
        left_head.append(&mut right_tail);
        document.blocks[start_line].spans = normalize_spans(left_head);
        document.blocks.drain(start_line + 1..=end_line);
    }

    for (idx, block) in document.blocks.iter_mut().enumerate() {
        block.source_line = idx;
        if block.spans.is_empty() {
            block.spans.push(Db8Span {
                text: String::new(),
                styles: StyleSet::default(),
            });
        }
    }

    true
}

fn delete_from_spans(spans: &[Db8Span], start: usize, end: usize) -> Vec<Db8Span> {
    let (left, rest) = split_spans_at(spans, start);
    let (_, right) = split_spans_at(&rest, end.saturating_sub(start));
    normalize_spans(left.into_iter().chain(right).collect())
}

fn rendered_range_has_inline_style(
    document: &Db8Document,
    start: usize,
    end: usize,
    style_token: &str,
) -> bool {
    copy_rendered_range(document, start, end).is_some_and(|fragment| {
        let mut saw_text = false;
        let all_styled = fragment
            .blocks
            .iter()
            .flat_map(|block| &block.spans)
            .filter(|span| !span.text.is_empty())
            .all(|span| {
                saw_text = true;
                match style_token {
                    "highlight" => span.styles.highlight,
                    "emphasis" => span.styles.emphasis,
                    "underline" => span.styles.underline,
                    "shrunk" => span.styles.shrunk,
                    _ => false,
                }
            });
        saw_text && all_styled
    })
}

fn apply_inline_style_to_rendered_range(
    document: &mut Db8Document,
    start: usize,
    end: usize,
    style_token: &str,
    enabled: bool,
) -> bool {
    if start >= end || document.blocks.is_empty() {
        return false;
    }

    let start_cursor = rendered_global_offset_to_cursor(document, start);
    let end_cursor = rendered_global_offset_to_cursor(document, end);
    let start_line = start_cursor
        .line
        .min(document.blocks.len().saturating_sub(1));
    let end_line = end_cursor.line.min(document.blocks.len().saturating_sub(1));

    for line in start_line..=end_line {
        let line_len = document
            .blocks
            .get(line)
            .map(rendered_line_text)
            .map_or(0, |text| text.len());
        let local_start = if line == start_line {
            start_cursor.column
        } else {
            0
        };
        let local_end = if line == end_line {
            end_cursor.column
        } else {
            line_len
        };
        if local_start < local_end
            && let Some(block) = document.blocks.get_mut(line)
        {
            block.spans =
                apply_style_to_spans(&block.spans, local_start, local_end, style_token, enabled);
        }
    }

    true
}

fn apply_style_to_spans(
    spans: &[Db8Span],
    start: usize,
    end: usize,
    style_token: &str,
    enabled: bool,
) -> Vec<Db8Span> {
    let (left, rest) = split_spans_at(spans, start);
    let (mut middle, right) = split_spans_at(&rest, end.saturating_sub(start));
    for span in &mut middle {
        set_inline_style(&mut span.styles, style_token, enabled);
    }
    normalize_spans(left.into_iter().chain(middle).chain(right).collect())
}

fn set_inline_style(styles: &mut StyleSet, style_token: &str, enabled: bool) {
    match style_token {
        "highlight" => styles.highlight = enabled,
        "emphasis" => styles.emphasis = enabled,
        "underline" => styles.underline = enabled,
        "shrunk" => styles.shrunk = enabled,
        _ => {}
    }
}

fn split_spans_at(spans: &[Db8Span], column: usize) -> (Vec<Db8Span>, Vec<Db8Span>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut consumed = 0usize;

    for span in spans {
        let span_len = span.text.len();
        let span_start = consumed;
        let span_end = consumed + span_len;

        if column <= span_start {
            right.push(span.clone());
        } else if column >= span_end {
            left.push(span.clone());
        } else {
            let local = clamp_to_char_boundary(&span.text, column - span_start);
            let (left_text, right_text) = span.text.split_at(local);
            if !left_text.is_empty() {
                left.push(Db8Span {
                    text: left_text.to_string(),
                    styles: span.styles,
                });
            }
            if !right_text.is_empty() {
                right.push(Db8Span {
                    text: right_text.to_string(),
                    styles: span.styles,
                });
            }
        }

        consumed = span_end;
    }

    (normalize_spans(left), normalize_spans(right))
}

fn normalize_document_spans(document: &mut Db8Document) {
    for block in &mut document.blocks {
        block.spans = normalize_spans(std::mem::take(&mut block.spans));
    }
}

fn normalize_spans(spans: Vec<Db8Span>) -> Vec<Db8Span> {
    let mut normalized: Vec<Db8Span> = Vec::new();
    for span in spans.into_iter().filter(|span| !span.text.is_empty()) {
        if let Some(last) = normalized.last_mut()
            && same_styles(last.styles, span.styles)
        {
            last.text.push_str(&span.text);
            continue;
        }
        normalized.push(span);
    }

    if normalized.is_empty() {
        normalized.push(Db8Span {
            text: String::new(),
            styles: StyleSet::default(),
        });
    }

    normalized
}

fn same_styles(a: StyleSet, b: StyleSet) -> bool {
    a.pocket == b.pocket
        && a.hat == b.hat
        && a.block == b.block
        && a.tag == b.tag
        && a.cite == b.cite
        && a.emphasis == b.emphasis
        && a.underline == b.underline
        && a.shrunk == b.shrunk
        && a.highlight == b.highlight
}

fn render_line_mouse_column(
    block: &Db8Block,
    mouse_x: f32,
    sidebar_open: bool,
    viewport_width: f32,
    style_config: &Db8StyleConfig,
) -> usize {
    const SIDEBAR_WIDTH: f32 = 240.0;
    const EDITOR_PADDING_X: f32 = 16.0;

    let document_left = if sidebar_open { SIDEBAR_WIDTH } else { 0.0 };
    let editor_width = (viewport_width - document_left - EDITOR_PADDING_X * 2.0).max(0.0);
    let text_width = rendered_block_width(block, style_config);
    let centered_offset = if matches!(
        block.style,
        BlockStyle::Pocket | BlockStyle::Hat | BlockStyle::Block
    ) {
        ((editor_width - text_width) / 2.0).max(0.0)
    } else {
        0.0
    };
    let text_left = document_left + EDITOR_PADDING_X + centered_offset;
    let target_x = mouse_x - text_left;
    if target_x <= 0.0 {
        return 0;
    }

    let mut column = 0usize;
    let mut x = 0.0f32;

    for span in &block.spans {
        let effective_styles = block_style_to_styles(block.style, span.styles);
        let font_px = rendered_span_font_size_px(effective_styles, style_config);
        let weight_factor = if effective_styles.pocket
            || effective_styles.hat
            || effective_styles.block
            || effective_styles.tag
            || effective_styles.emphasis
        {
            1.08
        } else {
            1.0
        };

        for ch in span.text.chars() {
            let width = approximate_calibri_char_width(ch, font_px) * weight_factor;
            if target_x < x + width / 2.0 {
                return column;
            }
            x += width;
            column += 1;
        }
    }

    column
}

fn rendered_block_width(block: &Db8Block, style_config: &Db8StyleConfig) -> f32 {
    block
        .spans
        .iter()
        .map(|span| {
            let effective_styles = block_style_to_styles(block.style, span.styles);
            let font_px = rendered_span_font_size_px(effective_styles, style_config);
            let weight_factor = if effective_styles.pocket
                || effective_styles.hat
                || effective_styles.block
                || effective_styles.tag
                || effective_styles.emphasis
            {
                1.08
            } else {
                1.0
            };
            span.text
                .chars()
                .map(|ch| approximate_calibri_char_width(ch, font_px) * weight_factor)
                .sum::<f32>()
        })
        .sum()
}

fn rendered_span_font_size_px(styles: StyleSet, style_config: &Db8StyleConfig) -> f32 {
    let base_size_pt = if styles.pocket {
        26.0
    } else if styles.hat {
        22.0
    } else if styles.block {
        16.0
    } else if styles.tag || styles.cite {
        13.0
    } else {
        11.0
    };

    let size_pt = if styles.shrunk {
        base_size_pt * style_config.shrunk.font_size_scale
    } else {
        base_size_pt
    };

    size_pt * 1.33
}

fn approximate_calibri_char_width(ch: char, font_px: f32) -> f32 {
    let factor = match ch {
        ' ' => 0.28,
        'i' | 'l' | 'I' | '!' | '|' | '.' | ',' | ';' | ':' | '\'' => 0.24,
        'f' | 'j' | 'r' | 't' | '(' | ')' | '[' | ']' => 0.34,
        'm' | 'w' | 'M' | 'W' | '@' | '%' => 0.82,
        'A'..='Z' => 0.62,
        '0'..='9' => 0.53,
        _ => 0.50,
    };
    font_px * factor
}

fn is_line_semantic_style(style_token: &str) -> bool {
    matches!(style_token, "pocket" | "hat" | "block" | "tag" | "cite")
}

fn block_style_from_token(style_token: &str) -> BlockStyle {
    match style_token {
        "pocket" => BlockStyle::Pocket,
        "hat" => BlockStyle::Hat,
        "block" => BlockStyle::Block,
        "tag" => BlockStyle::Tag,
        "cite" => BlockStyle::Cite,
        _ => BlockStyle::Normal,
    }
}

fn clear_primary_semantic_styles(styles: &mut StyleSet) {
    styles.pocket = false;
    styles.hat = false;
    styles.block = false;
    styles.tag = false;
    styles.cite = false;
}

fn semantic_block_class(block: &Db8Block) -> &'static str {
    match block.style {
        BlockStyle::Pocket => "pocket",
        BlockStyle::Hat => "hat",
        BlockStyle::Block => "block",
        BlockStyle::Tag => "tag",
        BlockStyle::Cite => "cite",
        BlockStyle::Normal => "normal",
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
        let effective_styles = block_style_to_styles(block.style, span.styles);
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
                    effective_styles,
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
                    effective_styles,
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
                        effective_styles,
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
                        effective_styles,
                        false,
                        style_config,
                        colors,
                    ));
                }
                if !middle.is_empty() {
                    segments.push(render_db8_text_segment(
                        middle,
                        effective_styles,
                        true,
                        style_config,
                        colors,
                    ));
                }
                if !right.is_empty() {
                    segments.push(render_db8_text_segment(
                        right,
                        effective_styles,
                        false,
                        style_config,
                        colors,
                    ));
                }
            }
        } else if !span.text.is_empty() {
            segments.push(render_db8_text_segment(
                &span.text,
                effective_styles,
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

fn block_style_to_styles(block_style: BlockStyle, mut styles: StyleSet) -> StyleSet {
    match block_style {
        BlockStyle::Pocket => styles.pocket = true,
        BlockStyle::Hat => styles.hat = true,
        BlockStyle::Block => styles.block = true,
        BlockStyle::Tag => styles.tag = true,
        BlockStyle::Cite => styles.cite = true,
        BlockStyle::Normal => {}
    }
    styles
}

fn render_db8_text_segment(
    text: &str,
    styles: StyleSet,
    selected: bool,
    style_config: &Db8StyleConfig,
    _colors: AppColors,
) -> AnyElement {
    let base_color: gpui::Rgba = gpui::black().into();
    let highlight_bg = color_from_hex(&style_config.highlight.background, gpui::yellow().into());
    let highlight_text = color_from_hex(&style_config.highlight.text_color, gpui::black().into());
    let tag_color = color_from_hex(&style_config.tag.text_color, gpui::black().into());
    let cite_color = color_from_hex(&style_config.cite.text_color, gpui::black().into());
    let base_size_pt = if styles.pocket {
        26.0
    } else if styles.hat {
        22.0
    } else if styles.block {
        16.0
    } else if styles.tag || styles.cite {
        13.0
    } else {
        11.0
    };
    let size_pt = if styles.shrunk {
        base_size_pt * style_config.shrunk.font_size_scale
    } else {
        base_size_pt
    };

    div()
        .id(SharedString::from(format!(
            "db8-{}",
            semantic_span_class(styles)
        )))
        .font_family("Calibri")
        .text_size(px(size_pt * 1.33))
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
            styles.pocket || styles.hat || styles.block || styles.tag || styles.emphasis,
            |this| this.font_weight(gpui::FontWeight::BLACK),
        )
        .when(styles.hat, |this| {
            this.underline().border_b_1().border_color(gpui::black())
        })
        .when(styles.block || (styles.underline && !styles.hat), |this| {
            this.underline()
        })
        .when(styles.highlight, |this| this.bg(highlight_bg))
        .when(styles.emphasis, |this| {
            this.border_1().border_color(gpui::black()).px_1()
        })
        .when(selected, |this| this.bg(gpui::blue().opacity(0.22)))
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

