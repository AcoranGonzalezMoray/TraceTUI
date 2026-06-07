use crate::app::ui::agents::constants::{
    MD_BOLD_DELIMITER, MD_BOLD_FALLBACK, MD_BULLET_CHAR, MD_BULLET_DASH, MD_BULLET_STAR,
    MD_CODE_BLOCK_CLOSE, MD_CODE_BLOCK_OPEN, MD_CODE_DELIMITER, MD_CODE_LINE_PREFIX,
    MD_COLLAPSE_MARKER, MD_H1_PREFIX, MD_H2_PREFIX, MD_H3_PREFIX, MD_HR_CHAR, MD_HR_LINE,
    MD_HR_MAX_WIDTH, MD_HR_STAR, MD_HR_UNDER, MD_HR_WIDTH_REDUCTION, MD_INLINE_CODE_FALLBACK,
    MD_INLINE_PADDING, MD_INDENT_PADDING, MD_NUMERIC_PADDING, MD_TABLE_INNER_SEP,
    MD_TABLE_SEP_MAX_WIDTH, MD_TRUNCATION_SUFFIX, MIN_WRAP_WIDTH,
};
use crate::app::ui::theme::THEME;
use ratatui::text::Span;
use ratatui::{
    style::{Modifier, Style},
    text::Line,
};

pub fn parse_inline(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut byte_idx = 0;

    while byte_idx < text.len() {
        let remaining = &text[byte_idx..];

        if let Some((content, next_idx)) = try_take_bold(text, byte_idx) {
            spans.push(bold_span(content));
            byte_idx = next_idx;
            continue;
        }
        if remaining.starts_with(MD_BOLD_DELIMITER) {
            spans.push(plain_span(MD_BOLD_FALLBACK));
            byte_idx += MD_BOLD_DELIMITER.len();
            continue;
        }
        if let Some((content, next_idx)) = try_take_code(text, byte_idx) {
            spans.push(code_span(content));
            byte_idx = next_idx;
            continue;
        }
        if remaining.starts_with(MD_CODE_DELIMITER) {
            spans.push(plain_span(MD_INLINE_CODE_FALLBACK));
            byte_idx += MD_CODE_DELIMITER.len_utf8();
            continue;
        }

        byte_idx = consume_plain_span(text, byte_idx, &mut spans);
    }

    spans
}

fn try_take_bold(text: &str, byte_idx: usize) -> Option<(&str, usize)> {
    let remaining = &text[byte_idx..];
    if !remaining.starts_with(MD_BOLD_DELIMITER) {
        return None;
    }
    let content_start = byte_idx + MD_BOLD_DELIMITER.len();
    let end = text[content_start..].find(MD_BOLD_DELIMITER)?;
    let content = &text[content_start..content_start + end];
    Some((content, content_start + end + MD_BOLD_DELIMITER.len()))
}

fn try_take_code(text: &str, byte_idx: usize) -> Option<(&str, usize)> {
    let remaining = &text[byte_idx..];
    if !remaining.starts_with(MD_CODE_DELIMITER) {
        return None;
    }
    let content_start = byte_idx + MD_CODE_DELIMITER.len_utf8();
    let end = text[content_start..].find(MD_CODE_DELIMITER)?;
    let content = &text[content_start..content_start + end];
    Some((content, content_start + end + MD_CODE_DELIMITER.len_utf8()))
}

fn bold_span(content: &str) -> Span<'static> {
    Span::styled(
        content.to_string(),
        Style::default()
            .fg(THEME.text_main)
            .add_modifier(Modifier::BOLD),
    )
}

fn code_span(content: &str) -> Span<'static> {
    Span::styled(content.to_string(), Style::default().fg(THEME.success))
}

fn plain_span(content: &str) -> Span<'static> {
    Span::styled(content.to_string(), Style::default().fg(THEME.text_main))
}

fn consume_plain_span(text: &str, byte_idx: usize, spans: &mut Vec<Span<'static>>) -> usize {
    let remaining = &text[byte_idx..];
    let next_bold = remaining.find(MD_BOLD_DELIMITER);
    let next_code = remaining.find(MD_CODE_DELIMITER);
    let next = match (next_bold, next_code) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => remaining.len(),
    };
    let end = byte_idx + next;
    if end > byte_idx {
        spans.push(plain_span(&text[byte_idx..end]));
    } else {
        spans.push(plain_span(&text[byte_idx..byte_idx + 1]));
        return byte_idx + 1;
    }
    end
}

pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let char_len = text.chars().count();
    if char_len <= width || width < MIN_WRAP_WIDTH {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;

    while start < chars.len() {
        if start + width >= chars.len() {
            result.push(chars[start..].iter().collect());
            break;
        }
        let mut split = start + width;
        if let Some(space_offset) = chars[start..split].iter().rposition(|c| c.is_whitespace()) {
            split = start + space_offset;
        }
        if split == start {
            split = (start + width).min(chars.len());
        }
        result.push(chars[start..split].iter().collect());
        start = split;
        while start < chars.len() && chars[start].is_whitespace() {
            start += 1;
        }
    }
    result
}

pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    let mut output: String = text.chars().take(max_chars).collect();
    if text.chars().count() > max_chars {
        output.push_str(MD_TRUNCATION_SUFFIX);
    }
    output
}

pub fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|')
        && trimmed.ends_with('|')
        && trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ' || c == '\t')
}

pub fn render_table_row(line: &str, is_header: bool) -> Vec<Line<'static>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return Vec::new();
    }
    let inner = trimmed.trim_matches('|');
    let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
    let mut spans = Vec::new();
    for (ci, cell) in cells.iter().enumerate() {
        if ci > 0 {
            spans.push(Span::styled(
                MD_TABLE_INNER_SEP,
                Style::default().fg(THEME.text_dim),
            ));
        }
        if is_header {
            let mut cell_spans = parse_inline(cell);
            for span in &mut cell_spans {
                span.style = span.style.add_modifier(Modifier::BOLD);
            }
            spans.extend(cell_spans);
        } else {
            spans.extend(parse_inline(cell));
        }
    }
    vec![Line::from(spans)]
}

pub fn render_to_lines(text: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    let mut in_code_block = false;
    let mut pending_header_row: Option<String> = None;

    for raw_line in text.lines() {
        let line = raw_line.trim_end();

        if let Some(next) = handle_code_fence(&mut lines, line, in_code_block) {
            in_code_block = next;
            continue;
        }
        if in_code_block {
            emit_code_line(&mut lines, line, width);
            continue;
        }
        if line.is_empty() {
            lines.push(Line::from(""));
            continue;
        }
        if let Some(_next) = handle_heading(&mut lines, line) {
            continue;
        }
        if handle_horizontal_rule(&mut lines, line, width) {
            continue;
        }
        if handle_table_separator(&mut lines, line, width, &mut pending_header_row) {
            continue;
        }
        if handle_table_row(&mut lines, line, &mut pending_header_row) {
            continue;
        }
        flush_pending_header(&mut lines, &mut pending_header_row);

        if handle_unordered_list(&mut lines, line, width) {
            continue;
        }
        if handle_ordered_list(&mut lines, line, width) {
            continue;
        }
        emit_paragraph(&mut lines, line, width);
    }

    lines
}

fn handle_code_fence(
    lines: &mut Vec<Line<'static>>,
    line: &str,
    in_code_block: bool,
) -> Option<bool> {
    if !line.starts_with("```") {
        return None;
    }
    let next = !in_code_block;
    let content = if next { MD_CODE_BLOCK_OPEN } else { MD_CODE_BLOCK_CLOSE };
    lines.push(Line::from(Span::styled(
        content,
        Style::default().fg(THEME.success),
    )));
    Some(next)
}

fn emit_code_line(lines: &mut Vec<Line<'static>>, line: &str, width: usize) {
    for wrapped in wrap(line, width.saturating_sub(MD_INDENT_PADDING).max(MIN_WRAP_WIDTH)) {
        lines.push(Line::from(Span::styled(
            format!("{}{} ", MD_CODE_LINE_PREFIX, wrapped),
            Style::default().fg(THEME.success),
        )));
    }
}

fn handle_heading(lines: &mut Vec<Line<'static>>, line: &str) -> Option<()> {
    if let Some(content) = line.strip_prefix(MD_H3_PREFIX) {
        lines.push(Line::from(Span::styled(
            format!(" {} ", content),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )));
        return Some(());
    }
    if let Some(content) = line.strip_prefix(MD_H2_PREFIX) {
        lines.push(Line::from(Span::styled(
            format!(" {} ", content),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )));
        return Some(());
    }
    if let Some(content) = line.strip_prefix(MD_H1_PREFIX) {
        lines.push(Line::from(Span::styled(
            format!(" {} ", content),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));
        return Some(());
    }
    None
}

fn handle_horizontal_rule(lines: &mut Vec<Line<'static>>, line: &str, width: usize) -> bool {
    if line != MD_HR_LINE && line != MD_HR_STAR && line != MD_HR_UNDER {
        return false;
    }
    let width = width
        .saturating_sub(MD_HR_WIDTH_REDUCTION)
        .min(MD_HR_MAX_WIDTH);
    lines.push(Line::from(Span::styled(
        MD_HR_CHAR.repeat(width),
        Style::default().fg(THEME.text_dim),
    )));
    true
}

fn handle_table_separator(
    lines: &mut Vec<Line<'static>>,
    line: &str,
    width: usize,
    pending_header_row: &mut Option<String>,
) -> bool {
    if !is_table_separator(line) {
        return false;
    }
    if let Some(header) = pending_header_row.take() {
        for row in render_table_row(&header, true) {
            lines.push(row);
        }
        let width = width
            .saturating_sub(MD_HR_WIDTH_REDUCTION)
            .min(MD_TABLE_SEP_MAX_WIDTH);
        lines.push(Line::from(Span::styled(
            MD_HR_CHAR.repeat(width),
            Style::default().fg(THEME.text_dim),
        )));
    }
    true
}

fn handle_table_row(
    lines: &mut Vec<Line<'static>>,
    line: &str,
    pending_header_row: &mut Option<String>,
) -> bool {
    if !line.trim_start().starts_with('|') || !line.trim_end().ends_with('|') {
        return false;
    }
    if pending_header_row.is_none() {
        *pending_header_row = Some(line.to_string());
        return true;
    }
    if let Some(header) = pending_header_row.take() {
        for row in render_table_row(&header, false) {
            lines.push(row);
        }
    }
    for row in render_table_row(line, false) {
        lines.push(row);
    }
    true
}

fn flush_pending_header(
    lines: &mut Vec<Line<'static>>,
    pending_header_row: &mut Option<String>,
) {
    if let Some(header) = pending_header_row.take() {
        for row in render_table_row(&header, false) {
            lines.push(row);
        }
    }
}

fn handle_unordered_list(lines: &mut Vec<Line<'static>>, line: &str, width: usize) -> bool {
    let content = if let Some(c) = line.strip_prefix(MD_BULLET_DASH) {
        c
    } else if let Some(c) = line.strip_prefix(MD_BULLET_STAR) {
        c
    } else {
        return false;
    };
    for wrapped in wrap(content, width.saturating_sub(MD_INDENT_PADDING).max(MIN_WRAP_WIDTH)) {
        let mut spans = vec![Span::styled(MD_BULLET_CHAR, Style::default().fg(THEME.primary))];
        spans.extend(parse_inline(&wrapped));
        lines.push(Line::from(spans));
    }
    true
}

fn handle_ordered_list(lines: &mut Vec<Line<'static>>, line: &str, width: usize) -> bool {
    if !(line.starts_with(|c: char| c.is_ascii_digit()) && line.len() > 2) {
        return false;
    }
    let dot_pos = line.find('.').unwrap_or(1);
    if line.as_bytes().get(dot_pos) != Some(&b'.')
        || !line
            .as_bytes()
            .get(dot_pos + 1)
            .is_some_and(|&b| b == b' ' || b == b'\t')
    {
        return false;
    }
    let num = &line[..dot_pos + 1];
    let content = &line[dot_pos + 1..].trim_start();
    for wrapped in wrap(content, width.saturating_sub(MD_NUMERIC_PADDING).max(MIN_WRAP_WIDTH)) {
        let mut spans = vec![Span::styled(
            format!(" {} ", num),
            Style::default().fg(THEME.primary),
        )];
        spans.extend(parse_inline(&wrapped));
        lines.push(Line::from(spans));
    }
    true
}

fn emit_paragraph(lines: &mut Vec<Line<'static>>, line: &str, width: usize) {
    for wrapped in wrap(line, width.saturating_sub(MD_INLINE_PADDING).max(MIN_WRAP_WIDTH)) {
        lines.push(Line::from(parse_inline(&wrapped)));
    }
}

pub fn collapse_lines(text: &str, collapse: bool, search_query: &str) -> Vec<String> {
    let mut source = Vec::new();
    let mut skip_section = false;
    let query = search_query.to_lowercase();

    for line in text.lines() {
        if collapse {
            if let Some(_next) = handle_collapsed_heading(&mut source, line, &mut skip_section) {
                continue;
            }
            if skip_section {
                continue;
            }
        }
        if !query.is_empty() && !line.to_lowercase().contains(&query) {
            continue;
        }
        source.push(line.to_string());
    }

    source
}

fn handle_collapsed_heading(
    source: &mut Vec<String>,
    line: &str,
    skip_section: &mut bool,
) -> Option<()> {
    if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
        source.push(format!("{}{}", line, MD_COLLAPSE_MARKER));
        *skip_section = true;
        Some(())
    } else {
        None
    }
}
