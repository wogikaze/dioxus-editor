use std::iter::repeat;

pub type LineId = u64;

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
