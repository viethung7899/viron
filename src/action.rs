use crate::editor;

#[derive(Debug)]
pub enum Action {
    Quit,

    Undo,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    MoveToLineStart,
    MoveToLineEnd,

    EnterMode(editor::Mode),
    SetWaitingCmd(char),

    InsertCharAtCursor(char),
    DeleteCharAtCursor,
    DeleteCurrentLine,
}

impl Action {
    pub fn execute(&self, editor: &mut editor::Editor) {
        let (_, viewport_height) = editor.get_viewport_size();
        match self {
            Action::MoveUp => {
                if editor.cursor.row == 0 {
                    editor.offset.row = editor.offset.row.saturating_sub(1);
                } else {
                    editor.cursor.row -= 1;
                }
            }
            Action::MoveDown => {
                if editor.cursor.row + 1 >= viewport_height {
                    editor.offset.row += 1;
                } else {
                    editor.cursor.row += 1;
                }
            }
            Action::MoveLeft => {
                editor.cursor.col = editor.cursor.col.saturating_sub(1);
            }
            Action::MoveRight => {
                editor.cursor.col += 1;
            }
            Action::PageUp => {
                let (_, height) = editor.get_viewport_size();
                editor.offset.row = editor.offset.row.saturating_sub(height);
            }
            Action::PageDown => {
                let (_, height) = editor.get_viewport_size();
                if editor.buffer.len() > (editor.offset.row + height) as usize {
                    editor.offset.row += height;
                }
            }
            Action::MoveToLineStart => {
                editor.cursor.col = 0;
            }
            Action::MoveToLineEnd => {
                editor.cursor.col = editor
                    .get_viewport_line(editor.cursor.row)
                    .map_or(0, |line| line.len() as u16)
            }
            Action::EnterMode(mode) => {
                editor.mode = *mode;
            }
            Action::InsertCharAtCursor(char) => {
                let line = editor.get_buffer_line_index();
                let offset = editor.cursor.col as usize;
                editor.buffer.insert(line as usize, offset, *char);
                editor.cursor.col += 1;
            }
            Action::DeleteCharAtCursor => {
                let line = editor.get_buffer_line_index();
                let offset = editor.cursor.col;
                editor.buffer.remove(line as usize, offset as usize);
            }
            Action::DeleteCurrentLine => {
                let line_index = editor.get_buffer_line_index();
                editor.buffer.remove_line(line_index as usize);
            }
            Action::SetWaitingCmd(char) => {
                editor.waiting_cmd = Some(*char);
            }
            Action::Undo => {}
            _ => {}
        }
    }
}
