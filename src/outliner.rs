use crate::model::{Document, Line, SelectionRange, char_to_byte_index};
use dioxus::events::FormData;
use dioxus::prelude::*;
use keyboard_types::{Key, Modifiers};
use std::ops::Range;

#[derive(Clone, PartialEq, Debug)]
enum InlineNode {
    Text(String),
    Bold(Vec<InlineNode>),
    Italic(Vec<InlineNode>),
    Strikethrough(Vec<InlineNode>),
    Underline(Vec<InlineNode>),
    Center(Vec<InlineNode>),
    Right(Vec<InlineNode>),
    Quote(Vec<InlineNode>),
    Code(String),
    Tag(String),
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum LineAlignment {
    None,
    Center,
    Right,
}

#[derive(Copy, Clone)]
enum MoveDirection {
    Up,
    Down,
}

trait CursorPositionExt {
    fn cursor_position(&self) -> usize;
}

impl CursorPositionExt for Event<FormData> {
    fn cursor_position(&self) -> usize {
        cursor_position_from_event(self)
    }
}

#[cfg(target_arch = "wasm32")]
fn cursor_position_from_event(evt: &Event<FormData>) -> usize {
    use wasm_bindgen::JsCast;
    use web_sys::{HtmlInputElement, HtmlTextAreaElement};

    let fallback = evt.value().chars().count();

    evt.data()
        .downcast::<web_sys::Event>()
        .and_then(|event| event.target())
        .and_then(|target| {
            target
                .dyn_ref::<HtmlInputElement>()
                .and_then(|input| input.selection_start().ok().flatten())
                .or_else(|| {
                    target
                        .dyn_ref::<HtmlTextAreaElement>()
                        .and_then(|textarea| textarea.selection_start().ok().flatten())
                })
        })
        .map(|pos| pos as usize)
        .unwrap_or(fallback)
}

#[cfg(not(target_arch = "wasm32"))]
fn cursor_position_from_event(evt: &Event<FormData>) -> usize {
    evt.value().chars().count()
}

#[derive(Props, Clone, PartialEq)]
pub struct OutlinerProps {
    pub initial_text: String,
}

#[component]
pub fn Outliner(props: OutlinerProps) -> Element {
    let document = use_signal(|| Document::from_text(&props.initial_text));
    let selection = use_signal(|| SelectionRange::caret(0, 0));

    rsx! {
        div { class: "outliner",
            for (index, line) in document.read().lines.iter().enumerate() {
                LineView {
                    key: "{line.id}",
                    line_index: index,
                    line: line.clone(),
                    document: document.clone(),
                    selection: selection.clone(),
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct LineViewProps {
    line_index: usize,
    line: Line,
    document: Signal<Document>,
    selection: Signal<SelectionRange>,
}

#[component]
fn LineView(props: LineViewProps) -> Element {
    let line_index = props.line_index;
    let document = props.document;
    let mut selection = props.selection;
    let line = props.line;

    let fallback_text_for_focus_len = line.text.chars().count();
    let fallback_text_for_click_len = fallback_text_for_focus_len;

    // Check if this line has the caret
    let is_focused = selection.read().focus.line == line_index;

    rsx! {
        div {
            class: "outliner-line",
            style: format!("margin-left: {}px;", line.indent * 16),
            span { class: "line-number", "{line_index + 1}" }
            
            // Show input only if this line is focused, otherwise show rendered view
            if is_focused {
                input {
                    class: "line-input",
                    value: line.text.clone(),
                    autofocus: true,
                    oninput: move |evt| {
                        handle_input(
                            evt.value(),
                            evt.cursor_position(),
                            line_index,
                            document,
                            selection,
                        );
                    },
                    onfocus: move |_| {
                        selection.set(SelectionRange::caret(
                            line_index,
                            fallback_text_for_focus_len,
                        ));
                    },
                    onclick: move |_| {
                        selection.set(SelectionRange::caret(
                            line_index,
                            fallback_text_for_click_len,
                        ));
                    },
                    onkeydown: move |evt| {
                        handle_keydown(evt, line_index, document, selection);
                    },
                }
            } else {
                // Render view is clickable to set focus
                div {
                    class: "line-render-container",
                    onclick: move |_| {
                        selection.set(SelectionRange::caret(line_index, 0));
                    },
                    {render_line(line_index, &line, document.clone())}
                }
            }
        }
    }
}

fn render_line(line_index: usize, line: &Line, document: Signal<Document>) -> Element {
    if let Some((level, content)) = parse_heading_line(&line.text) {
        let nodes = parse_inline_nodes(content);
        let rendered_nodes = render_inline(&nodes);

        return match level {
            1 => rsx! { h1 { class: "line-render", {rendered_nodes} } },
            2 => rsx! { h2 { class: "line-render", {rendered_nodes} } },
            3 => rsx! { h3 { class: "line-render", {rendered_nodes} } },
            4 => rsx! { h4 { class: "line-render", {rendered_nodes} } },
            5 => rsx! { h5 { class: "line-render", {rendered_nodes} } },
            _ => rsx! { h6 { class: "line-render", {rendered_nodes} } },
        };
    }

    if let Some((checked, content)) = parse_checkbox_line(&line.text) {
        let nodes = parse_inline_nodes(content);
        let toggle_handle = document.clone();

        return rsx! {
            div { class: "line-render",
                input {
                    r#type: "checkbox",
                    checked: checked,
                    onclick: move |_| toggle_checkbox(line_index, toggle_handle.clone()),
                }
                span { {render_inline(&nodes)} }
            }
        };
    }

    let (alignment, content) = parse_alignment_prefix(&line.text);
    let nodes = parse_inline_nodes(content);
    let alignment_style = match alignment {
        LineAlignment::None => String::new(),
        LineAlignment::Center => "text-align: center;".to_string(),
        LineAlignment::Right => "text-align: right;".to_string(),
    };

    rsx! {
        div { class: "line-render", style: alignment_style, {render_inline(&nodes)} }
    }
}

fn parse_alignment_prefix(text: &str) -> (LineAlignment, &str) {
    for (marker, alignment) in [
        ("[| ", LineAlignment::Center),
        ("[> ", LineAlignment::Right),
    ] {
        if let Some(rest) = text.strip_prefix(marker) {
            if let Some(end_idx) = rest.rfind(']') {
                return (alignment, &rest[..end_idx]);
            }
        }
    }

    (LineAlignment::None, text)
}

fn parse_checkbox_line(text: &str) -> Option<(bool, &str)> {
    let (checked, rest) = if let Some(rest) = text.strip_prefix("[ ]") {
        (false, rest)
    } else if let Some(rest) = text
        .strip_prefix("[x]")
        .or_else(|| text.strip_prefix("[X]"))
    {
        (true, rest)
    } else {
        return None;
    };

    let content = rest.strip_prefix(' ').unwrap_or(rest);
    Some((checked, content))
}

fn parse_heading_line(text: &str) -> Option<(usize, &str)> {
    let rest = text.strip_prefix('[')?;
    let hash_count = rest.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&hash_count) {
        return None;
    }

    let after_hash = &rest[hash_count..];
    let after_space = after_hash.strip_prefix(' ')?;
    let closing = after_space.rfind(']')?;
    Some((hash_count, &after_space[..closing]))
}

fn parse_inline_nodes(text: &str) -> Vec<InlineNode> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut index = 0usize;
    let (nodes, _) = parse_inline_range(text, &chars, &mut index, false);
    nodes
}

fn parse_inline_range(
    text: &str,
    chars: &[(usize, char)],
    index: &mut usize,
    stop_at_closing: bool,
) -> (Vec<InlineNode>, bool) {
    let mut nodes = Vec::new();
    let mut buffer = String::new();
    let mut closed = false;

    while *index < chars.len() {
        let ch = chars[*index].1;
        if stop_at_closing && ch == ']' {
            closed = true;
            *index += 1;
            break;
        }

        match ch {
            '`' => {
                flush_buffer(&mut buffer, &mut nodes);
                *index += 1;
                let code_start = *index;
                while *index < chars.len() && chars[*index].1 != '`' {
                    *index += 1;
                }

                if *index < chars.len() {
                    let start_byte = chars
                        .get(code_start)
                        .map(|(byte, _)| *byte)
                        .unwrap_or(text.len());
                    let end_byte = chars
                        .get(*index)
                        .map(|(byte, _)| *byte)
                        .unwrap_or(text.len());
                    let content = text.get(start_byte..end_byte).unwrap_or("");
                    nodes.push(InlineNode::Code(content.to_string()));
                    *index += 1;
                } else {
                    let start_byte = if code_start == 0 {
                        0
                    } else {
                        chars
                            .get(code_start - 1)
                            .map(|(byte, _)| *byte)
                            .unwrap_or(0)
                    };
                    buffer.push_str(&text[start_byte..]);
                    *index = chars.len();
                }
            }
            '[' => {
                flush_buffer(&mut buffer, &mut nodes);
                if let Some((node, new_index)) = parse_bracket_node(text, chars, *index) {
                    nodes.push(node);
                    *index = new_index;
                } else {
                    buffer.push('[');
                    *index += 1;
                }
            }
            '#' => {
                let is_start = *index == 0 || chars[*index - 1].1.is_whitespace();
                if is_start {
                    let tag_start_byte = chars[*index].0;
                    *index += 1;
                    let mut end = *index;
                    while end < chars.len() && !chars[end].1.is_whitespace() {
                        end += 1;
                    }

                    let tag_end_byte = if end < chars.len() {
                        chars[end].0
                    } else {
                        text.len()
                    };

                    let tag_text = &text[tag_start_byte + 1..tag_end_byte];
                    if tag_text.is_empty() {
                        buffer.push('#');
                        *index = end;
                    } else {
                        flush_buffer(&mut buffer, &mut nodes);
                        nodes.push(InlineNode::Tag(tag_text.to_string()));
                        *index = end;
                    }
                } else {
                    buffer.push('#');
                    *index += 1;
                }
            }
            _ => {
                buffer.push(ch);
                *index += 1;
            }
        }
    }

    flush_buffer(&mut buffer, &mut nodes);
    (nodes, closed)
}

fn flush_buffer(buffer: &mut String, nodes: &mut Vec<InlineNode>) {
    if !buffer.is_empty() {
        nodes.push(InlineNode::Text(std::mem::take(buffer)));
    }
}

fn parse_bracket_node(
    text: &str,
    chars: &[(usize, char)],
    index: usize,
) -> Option<(InlineNode, usize)> {
    if index + 2 >= chars.len() {
        return None;
    }

    let marker = chars[index + 1].1;
    if !matches!(marker, '*' | '/' | '-' | '_' | '|' | '>' | '"') {
        return None;
    }

    if chars[index + 2].1 != ' ' {
        return None;
    }

    let mut inner_index = index + 3;
    let (children, closed) = parse_inline_range(text, chars, &mut inner_index, true);
    if !closed {
        return None;
    }

    let node = match marker {
        '*' => InlineNode::Bold(children),
        '/' => InlineNode::Italic(children),
        '-' => InlineNode::Strikethrough(children),
        '_' => InlineNode::Underline(children),
        '|' => InlineNode::Center(children),
        '>' => InlineNode::Right(children),
        '"' => InlineNode::Quote(children),
        _ => return None,
    };

    Some((node, inner_index))
}

fn render_nodes(nodes: &[InlineNode]) -> Vec<Element> {
    nodes.iter().map(render_node).collect()
}

fn render_inline(nodes: &[InlineNode]) -> Element {
    let children = render_nodes(nodes);
    rsx! { Fragment { for child in children { {child} } } }
}

fn render_node(node: &InlineNode) -> Element {
    match node {
        InlineNode::Text(text) => rsx! { span { "{text}" } },
        InlineNode::Bold(children) => rsx! { strong { {render_inline(children)} } },
        InlineNode::Italic(children) => rsx! { em { {render_inline(children)} } },
        InlineNode::Strikethrough(children) => rsx! { del { {render_inline(children)} } },
        InlineNode::Underline(children) => {
            rsx! { span { style: "text-decoration: underline;", {render_inline(children)} } }
        }
        InlineNode::Center(children) => {
            rsx! { span { style: "display: block; text-align: center;", {render_inline(children)} } }
        }
        InlineNode::Right(children) => {
            rsx! { span { style: "display: block; text-align: right;", {render_inline(children)} } }
        }
        InlineNode::Quote(children) => rsx! { q { {render_inline(children)} } },
        InlineNode::Code(code) => rsx! { code { "{code}" } },
        InlineNode::Tag(tag) => rsx! { span { class: "tag", "#{tag}" } },
    }
}

fn toggle_checkbox(line_index: usize, mut document: Signal<Document>) {
    if let Some(line) = document.write().lines.get_mut(line_index) {
        if line.text.starts_with("[ ]") {
            line.text = line.text.replacen("[ ]", "[x]", 1);
        } else if line.text.starts_with("[x]") {
            line.text = line.text.replacen("[x]", "[ ]", 1);
        } else if line.text.starts_with("[X]") {
            line.text = line.text.replacen("[X]", "[ ]", 1);
        }
    }
}

fn handle_input(
    new_text: String,
    cursor_pos_bytes: usize,
    line_index: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
) {
    let cursor_pos_bytes = cursor_pos_bytes.min(new_text.len());
    let cursor_pos_chars = new_text[..cursor_pos_bytes].chars().count();

    if let Some(line) = document.write().lines.get_mut(line_index) {
        line.text = new_text;
    }

    selection.set(SelectionRange::caret(line_index, cursor_pos_chars));
}

fn handle_keydown(
    event: KeyboardEvent,
    line_index: usize,
    document: Signal<Document>,
    selection: Signal<SelectionRange>,
) {
    let key = event.data().key();
    let modifiers = event.data().modifiers();
    let fallback_text = document
        .read()
        .lines
        .get(line_index)
        .map(|line| line.text.clone())
        .unwrap_or_default();
    let caret_column = current_caret_column(line_index, selection, fallback_text.chars().count());

    if key == Key::Enter && modifiers.contains(Modifiers::SHIFT) {
        event.prevent_default();
        insert_root_line(line_index, document, selection);
        return;
    }

    if key == Key::Tab && modifiers.contains(Modifiers::SHIFT) {
        event.prevent_default();
        adjust_indent(document, selection, false);
        return;
    }

    match key {
        Key::Tab => {
            event.prevent_default();
            adjust_indent(document, selection, true);
        }
        Key::ArrowLeft => {
            if modifiers.contains(Modifiers::CONTROL) {
                event.prevent_default();
                adjust_indent(document, selection, false);
            }
        }
        Key::ArrowRight => {
            if modifiers.contains(Modifiers::CONTROL) {
                event.prevent_default();
                adjust_indent(document, selection, true);
            }
        }
        Key::ArrowUp | Key::ArrowDown => {
            let direction = if key == Key::ArrowUp {
                MoveDirection::Up
            } else {
                MoveDirection::Down
            };

            if modifiers.contains(Modifiers::ALT) && modifiers.contains(Modifiers::SHIFT) {
                event.prevent_default();
                duplicate_subtree(line_index, document, selection, direction, caret_column);
            } else if modifiers.contains(Modifiers::ALT) {
                event.prevent_default();
                move_subtree(line_index, document, selection, direction, caret_column);
            } else if modifiers.contains(Modifiers::CONTROL) {
                event.prevent_default();
                move_single_line(line_index, document, selection, direction, caret_column);
            } else {
                // Basic arrow navigation without modifiers
                event.prevent_default();
                move_caret_vertical(line_index, document, selection, direction, caret_column);
            }
        }
        Key::Enter => {
            event.prevent_default();
            handle_enter(line_index, caret_column, document, selection);
        }
        Key::Backspace => {
            if caret_column == 0 {
                event.prevent_default();
                handle_backspace(line_index, document, selection);
            }
        }
        Key::Character(ref ch) if ch == " " => {
            if caret_column == 0 {
                event.prevent_default();
                adjust_indent(document, selection, true);
            }
        }
        _ => {}
    }
}

fn current_caret_column(
    line_index: usize,
    selection: Signal<SelectionRange>,
    fallback_len: usize,
) -> usize {
    let selection_state = selection.read();
    if selection_state.focus.line == line_index {
        selection_state.focus.column
    } else {
        fallback_len
    }
}

fn handle_enter(
    line_index: usize,
    caret_column: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
) {
    let mut doc = document.write();

    let new_id = doc.next_line_id();

    if let Some(line) = doc.lines.get_mut(line_index) {
        let split_at = char_to_byte_index(&line.text, caret_column);
        let trailing = line.text.split_off(split_at);
        let indent = line.indent;
        let new_line = Line {
            id: new_id,
            indent,
            text: trailing,
            collapsed: false,
        };

        doc.lines.insert(line_index + 1, new_line);
    }

    drop(doc);

    selection.set(SelectionRange::caret(line_index + 1, 0));
}

fn handle_backspace(
    line_index: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
) {
    let mut new_selection = None;

    {
        let mut doc = document.write();

        if doc.lines.is_empty() || line_index >= doc.lines.len() {
            return;
        }

        if let Some(line) = doc.lines.get_mut(line_index) {
            if line.indent > 0 {
                line.indent = line.indent.saturating_sub(1);
                new_selection = Some(SelectionRange::caret(line_index, 0));
            } else if line_index > 0 {
                let current_line = doc.lines.remove(line_index);
                let previous_line = &mut doc.lines[line_index - 1];
                let previous_length = previous_line.text.chars().count();
                previous_line.text.push_str(&current_line.text);

                new_selection = Some(SelectionRange::caret(line_index - 1, previous_length));
            }
        }
    }

    if let Some(selection_range) = new_selection {
        selection.set(selection_range);
    }
}

fn adjust_indent(
    mut document: Signal<Document>,
    selection: Signal<SelectionRange>,
    increase: bool,
) {
    let selection_range = selection.read().clone();
    let line_range = selected_line_range(&document.read(), &selection_range);

    let mut doc = document.write();
    for index in line_range {
        if let Some(line) = doc.lines.get_mut(index) {
            if increase {
                line.indent = line.indent.saturating_add(1);
            } else {
                line.indent = line.indent.saturating_sub(1);
            }
        }
    }
}

fn selected_line_range(document: &Document, selection: &SelectionRange) -> Range<usize> {
    if document.lines.is_empty() {
        return 0..0;
    }

    let (start, end) = selection.normalized();
    let start_line = start.line.min(document.lines.len() - 1);
    let end_line = end.line.min(document.lines.len() - 1);

    start_line..(end_line + 1)
}

fn subtree_range(lines: &[Line], start_index: usize) -> Range<usize> {
    let base_indent = lines[start_index].indent;
    let mut end = start_index + 1;
    while end < lines.len() && lines[end].indent > base_indent {
        end += 1;
    }
    start_index..end
}

fn clamp_caret_column(document: &Document, line_index: usize, desired_column: usize) -> usize {
    document
        .lines
        .get(line_index)
        .map(|line| desired_column.min(line.text.chars().count()))
        .unwrap_or(0)
}

fn move_single_line(
    line_index: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
    direction: MoveDirection,
    caret_column: usize,
) {
    let new_index = {
        let mut doc = document.write();
        if doc.lines.is_empty() || line_index >= doc.lines.len() {
            None
        } else {
            match direction {
                MoveDirection::Up => {
                    if line_index > 0 {
                        doc.lines.swap(line_index, line_index - 1);
                        Some(line_index - 1)
                    } else {
                        None
                    }
                }
                MoveDirection::Down => {
                    if line_index + 1 < doc.lines.len() {
                        doc.lines.swap(line_index, line_index + 1);
                        Some(line_index + 1)
                    } else {
                        None
                    }
                }
            }
        }
    };

    if let Some(new_index) = new_index {
        let clamped_column = clamp_caret_column(&document.read(), new_index, caret_column);
        selection.set(SelectionRange::caret(new_index, clamped_column));
    }
}

fn move_subtree(
    line_index: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
    direction: MoveDirection,
    caret_column: usize,
) {
    let new_index = {
        let mut doc = document.write();
        if line_index >= doc.lines.len() {
            return;
        }

        let range = subtree_range(&doc.lines, line_index);
        let base_indent = doc.lines[line_index].indent;

        match direction {
            MoveDirection::Up => {
                let mut search = range.start as isize - 1;
                let mut previous_start = None;
                while search >= 0 {
                    let idx = search as usize;
                    let indent = doc.lines[idx].indent;
                    if indent < base_indent {
                        break;
                    }
                    if indent == base_indent {
                        previous_start = Some(idx);
                        break;
                    }
                    search -= 1;
                }

                let Some(prev_start) = previous_start else {
                    return;
                };
                let block: Vec<Line> = doc.lines.drain(range.clone()).collect();
                doc.lines.splice(prev_start..prev_start, block);
                prev_start
            }
            MoveDirection::Down => {
                let mut next_start = None;
                for idx in range.end..doc.lines.len() {
                    let indent = doc.lines[idx].indent;
                    if indent < base_indent {
                        break;
                    }
                    if indent == base_indent {
                        next_start = Some(idx);
                        break;
                    }
                }

                let Some(next_start) = next_start else {
                    return;
                };

                let next_range = subtree_range(&doc.lines, next_start);
                let block: Vec<Line> = doc.lines.drain(range.clone()).collect();
                let adjusted_next_start = next_start.saturating_sub(range.len());
                let insert_at = adjusted_next_start + next_range.len();
                doc.lines.splice(insert_at..insert_at, block);
                insert_at
            }
        }
    };

    let clamped_column = clamp_caret_column(&document.read(), new_index, caret_column);
    selection.set(SelectionRange::caret(new_index, clamped_column));
}

fn duplicate_subtree(
    line_index: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
    direction: MoveDirection,
    caret_column: usize,
) {
    let insert_at = {
        let mut doc = document.write();
        if line_index >= doc.lines.len() {
            return;
        }

        let range = subtree_range(&doc.lines, line_index);
        let new_block: Vec<Line> = doc.lines[range.clone()]
            .to_vec()
            .iter()
            .map(|line| {
                let mut new_line = line.clone();
                new_line.id = doc.next_line_id();
                new_line
            })
            .collect();

        match direction {
            MoveDirection::Up => {
                let insert_at = range.start;
                doc.lines.splice(insert_at..insert_at, new_block);
                insert_at
            }
            MoveDirection::Down => {
                let insert_at = range.end;
                doc.lines.splice(insert_at..insert_at, new_block);
                insert_at
            }
        }
    };

    let clamped_column = clamp_caret_column(&document.read(), insert_at, caret_column);
    selection.set(SelectionRange::caret(insert_at, clamped_column));
}

fn insert_root_line(
    line_index: usize,
    mut document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
) {
    let new_index = {
        let mut doc = document.write();
        let insert_at = line_index + 1;

        let new_line = Line {
            id: doc.next_line_id(),
            indent: 0,
            text: String::new(),
            collapsed: false,
        };

        doc.lines.insert(insert_at, new_line);
        insert_at
    };

    selection.set(SelectionRange::caret(new_index, 0));
}

fn move_caret_vertical(
    line_index: usize,
    document: Signal<Document>,
    mut selection: Signal<SelectionRange>,
    direction: MoveDirection,
    caret_column: usize,
) {
    let doc = document.read();
    let new_line_index = match direction {
        MoveDirection::Up => {
            if line_index > 0 {
                line_index - 1
            } else {
                return; // Already at first line
            }
        }
        MoveDirection::Down => {
            if line_index + 1 < doc.lines.len() {
                line_index + 1
            } else {
                return; // Already at last line
            }
        }
    };

    // Clamp the column to the new line's length
    let clamped_column = clamp_caret_column(&doc, new_line_index, caret_column);
    drop(doc);
    
    selection.set(SelectionRange::caret(new_line_index, clamped_column));
}
