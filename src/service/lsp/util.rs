use crate::core::document::Document;
use lsp_types::{Position, Range, TextDocumentContentChangeEvent, Uri};
use similar::{Algorithm, DiffOp, TextDiff};
use std::str::FromStr;

pub fn calculate_changes(old_text: &str, new_text: &str) -> Vec<TextDocumentContentChangeEvent> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .timeout(std::time::Duration::from_secs(1))
        .diff_chars(old_text, new_text);

    let mut changes = Vec::new();
    let mut current_change = String::new();
    let mut start_offset = 0;
    let mut old_offset = 0;

    for op in diff.grouped_ops(3).iter().flatten() {
        match op {
            DiffOp::Equal { old_index, len, .. } => {
                flush_insert(old_text, &mut changes, &mut current_change, start_offset);
                old_offset = old_index + len;
            }
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                flush_insert(old_text, &mut changes, &mut current_change, start_offset);
                let start_pos = calculate_position(old_text, *old_index);
                let end_pos = calculate_position(old_text, old_index + old_len);
                changes.push(TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: start_pos,
                        end: end_pos,
                    }),
                    range_length: None,
                    text: String::new(),
                });

                start_offset = old_index + old_len;
                old_offset = old_index + old_len;
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                if current_change.is_empty() {
                    start_offset = old_offset;
                }
                let text = new_text
                    .chars()
                    .skip(*new_index)
                    .take(*new_len)
                    .collect::<String>();
                current_change.push_str(&text);
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                flush_insert(old_text, &mut changes, &mut current_change, start_offset);
                let start_pos = calculate_position(old_text, *old_index);
                let end_pos = calculate_position(old_text, old_index + old_len);
                let text = new_text
                    .chars()
                    .skip(*new_index)
                    .take(*new_len)
                    .collect::<String>();
                changes.push(TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: start_pos,
                        end: end_pos,
                    }),
                    range_length: None,
                    text,
                });
                start_offset = old_index + old_len;
                old_offset = old_index + old_len;
            }
        }
    }

    flush_insert(old_text, &mut changes, &mut current_change, start_offset);

    changes
}

fn flush_insert(
    old_text: &str,
    changes: &mut Vec<TextDocumentContentChangeEvent>,
    mut current_change: &mut String,
    start_offset: usize,
) {
    if !current_change.is_empty() {
        let start_pos = calculate_position(old_text, start_offset);
        changes.push(TextDocumentContentChangeEvent {
            range: Some(Range {
                start: start_pos,
                end: start_pos,
            }),
            range_length: None,
            text: std::mem::take(&mut current_change),
        });
    }
}

fn calculate_position(text: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;

    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }

    Position { line, character }
}

pub fn get_uri_from_document(document: &Document) -> Option<Uri> {
    let path = document.full_file_path()?;
    Uri::from_str(path.to_str()?).ok()
}
