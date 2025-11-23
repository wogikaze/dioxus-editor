use crate::model::{Document, Line, SelectionRange, char_to_byte_index};
use dioxus::events::FormData;
use dioxus::prelude::*;
use keyboard_types::{Key, Modifiers};
use std::ops::Range;

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
                .dyn_into::<HtmlInputElement>()
                .ok()
                .and_then(|input| input.selection_start().ok().flatten())
                .or_else(|| {
                    target
                        .dyn_into::<HtmlTextAreaElement>()
                        .ok()
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

    rsx! {
        div {
            class: "outliner-line",
            style: format!("margin-left: {}px;", line.indent * 16),
            span { class: "line-number", "{line_index + 1}" }
            input {
                class: "line-input",
                value: line.text.clone(),
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
