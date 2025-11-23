use std::iter::repeat;

pub type LineId = u64;

#[derive(Clone, PartialEq, Debug)]
pub struct CaretPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SelectionRange {
    pub anchor: CaretPosition,
    pub focus: CaretPosition,
}

impl SelectionRange {
    pub fn caret(line: usize, column: usize) -> Self {
        let position = CaretPosition { line, column };
        Self {
            anchor: position.clone(),
            focus: position,
        }
    }

    pub fn normalized(&self) -> (CaretPosition, CaretPosition) {
        if self.anchor.line < self.focus.line
            || (self.anchor.line == self.focus.line && self.anchor.column <= self.focus.column)
        {
            (self.anchor.clone(), self.focus.clone())
        } else {
            (self.focus.clone(), self.anchor.clone())
        }
    }

    #[allow(dead_code)]
    pub fn is_collapsed(&self) -> bool {
        self.anchor.line == self.focus.line && self.anchor.column == self.focus.column
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Line {
    pub id: LineId,
    pub indent: u32,
    pub text: String,
    pub collapsed: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Document {
    pub lines: Vec<Line>,
}

impl Document {
    pub fn from_text(text: &str) -> Self {
        let lines = text
            .split('\n')
            .enumerate()
            .map(|(index, raw_line)| {
                let (indent, content) = split_indent(raw_line);
                Line {
                    id: index as LineId,
                    indent,
                    text: content.to_string(),
                    collapsed: false,
                }
            })
            .collect();

        Self { lines }
    }

    #[allow(dead_code)]
    pub fn to_text(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                format!(
                    "{}{}",
                    repeat('\t').take(line.indent as usize).collect::<String>(),
                    line.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn next_line_id(&self) -> LineId {
        self.lines
            .iter()
            .map(|line| line.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }
}

pub fn char_to_byte_index(text: &str, column: usize) -> usize {
    text.char_indices()
        .map(|(idx, _)| idx)
        .nth(column)
        .unwrap_or_else(|| text.len())
}

#[allow(dead_code)]
pub fn utf16_to_char_index(text: &str, utf16_offset: usize) -> usize {
    let mut utf16_count = 0usize;
    let mut char_index = 0usize;

    for ch in text.chars() {
        if utf16_count >= utf16_offset {
            break;
        }

        utf16_count += ch.len_utf16();
        char_index += 1;
    }

    char_index
}

fn split_indent(line: &str) -> (u32, &str) {
    let mut indent = 0u32;
    let mut byte_index = 0usize;

    for (idx, ch) in line.char_indices() {
        if matches!(ch, ' ' | '\t' | '　') {
            indent += 1;
            byte_index = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    (indent, &line[byte_index..])
}
