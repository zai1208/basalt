use core::fmt;

use tui_textarea::{Input, TextArea};

// TODO: Text wrapping according to the available width of the area
#[derive(Clone, Debug, Default)]
pub struct TextBuffer<'a> {
    textarea: TextArea<'a>,
    modified: bool,
}

#[derive(Clone, Debug)]
pub enum CursorMove {
    Top,
    Bottom,
    WordForward,
    WordBackward,
    Up,
    Down,
    Left,
    Right,
    Move(i32, i32),
    Jump(u16, u16),
}

impl From<(i32, i32)> for CursorMove {
    fn from(value: (i32, i32)) -> Self {
        Self::Move(value.0, value.1)
    }
}

impl fmt::Display for TextBuffer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw_buffer = self.textarea.lines().join("\n");
        write!(f, "{raw_buffer}")
    }
}

impl<'a> AsMut<TextBuffer<'a>> for TextBuffer<'a> {
    fn as_mut(&mut self) -> &mut TextBuffer<'a> {
        self
    }
}

impl From<String> for TextBuffer<'_> {
    fn from(value: String) -> Self {
        Self {
            textarea: value.lines().into(),
            ..Default::default()
        }
    }
}

impl<'a> From<&str> for TextBuffer<'a> {
    fn from(value: &str) -> Self {
        Self {
            textarea: value.lines().into(),
            ..Default::default()
        }
    }
}

impl<'a> TextBuffer<'a> {
    pub fn new(source: &str) -> Self {
        Self {
            textarea: source.lines().into(),
            ..Default::default()
        }
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn is_empty(&self) -> bool {
        self.textarea.is_empty()
    }

    pub fn with_cursor_position(mut self, (row, col): (usize, usize)) -> Self {
        let textarea = self.textarea_as_mut();
        textarea.move_cursor(tui_textarea::CursorMove::Jump(row as u16, col as u16));
        self
    }

    pub fn textarea_as_mut(&mut self) -> &mut TextArea<'a> {
        &mut self.textarea
    }

    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    pub fn modified(&self) -> bool {
        self.modified
    }

    pub fn edit(&mut self, input: Input) {
        self.modified = self.textarea.input(input);
    }

    pub fn cursor_move(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            CursorMove::Top => self.textarea.move_cursor(tui_textarea::CursorMove::Top),
            CursorMove::Bottom => self.textarea.move_cursor(tui_textarea::CursorMove::Bottom),
            CursorMove::Up => self.textarea.move_cursor(tui_textarea::CursorMove::Up),
            CursorMove::Down => self.textarea.move_cursor(tui_textarea::CursorMove::Down),
            CursorMove::Left => self.textarea.move_cursor(tui_textarea::CursorMove::Back),
            CursorMove::Right => self.textarea.move_cursor(tui_textarea::CursorMove::Forward),
            CursorMove::WordForward => self
                .textarea
                .move_cursor(tui_textarea::CursorMove::WordForward),
            CursorMove::WordBackward => self
                .textarea
                .move_cursor(tui_textarea::CursorMove::WordBack),
            CursorMove::Jump(row, col) => self
                .textarea
                .move_cursor(tui_textarea::CursorMove::Jump(row, col)),
            CursorMove::Move(row, col) => {
                let (cur_row, cur_col) = self.cursor();

                let row = match row.is_positive() {
                    true => cur_row.saturating_add(row as usize),
                    false => cur_row.saturating_sub(row.unsigned_abs() as usize),
                };

                let col = match col.is_positive() {
                    true => cur_col.saturating_add(col as usize),
                    false => cur_col.saturating_sub(col.unsigned_abs() as usize),
                };

                self.textarea
                    .move_cursor(tui_textarea::CursorMove::Jump(row as u16, col as u16))
            }
        }
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.textarea.cursor()
    }
}
