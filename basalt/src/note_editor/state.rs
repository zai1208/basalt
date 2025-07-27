use core::fmt;

use std::{
    fs::File,
    io::{self, Write},
    ops::RangeBounds,
    path::PathBuf,
    slice::SliceIndex,
};

use ratatui::widgets::ScrollbarState;
use tui_textarea::Input;

use super::{markdown_parser, text_buffer::CursorMove, TextBuffer};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Scrollbar {
    pub state: ScrollbarState,
    pub position: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    Read,
    View,
    Edit,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::View => write!(f, "VIEW"),
            Mode::Edit => write!(f, "EDIT"),
            Mode::Read => write!(f, "READ"),
        }
    }
}

// TODO: Two editing modes
// 1. Obsidian (Partial editing)
// 2. Full editing
// 3. Command mode
//
// TODO:
// - Better movement
// - Vim mode
// - Command mode to open a different text editor like Neovim or helix
#[derive(Clone, Debug, Default)]
pub struct EditorState<'text_buffer> {
    pub mode: Mode,
    text_buffer: TextBuffer<'text_buffer>,
    content: String,
    content_original: String,
    path: PathBuf,
    nodes: Vec<markdown_parser::Node>,
    scrollbar: Scrollbar,
    pub current_row: usize,
    // TODO: This can be utilized after toast implementation
    // error_message: Option<String>,
    active: bool,
    pub modified: bool,
    dirty: bool,
}

impl<'text_buffer> EditorState<'text_buffer> {
    pub fn content_slice<R>(&self, range: R) -> &str
    where
        R: RangeBounds<usize> + SliceIndex<str, Output = str>,
    {
        &self.content[range]
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_editing(&self) -> bool {
        self.mode == Mode::Edit
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn nodes(&self) -> &[markdown_parser::Node] {
        self.nodes.as_slice()
    }

    pub fn nodes_as_mut(&mut self) -> &mut [markdown_parser::Node] {
        self.nodes.as_mut_slice()
    }

    pub fn scrollbar(&self) -> &Scrollbar {
        &self.scrollbar
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn new(content: &str, path: PathBuf) -> Self {
        Self {
            nodes: markdown_parser::from_str(content),
            content_original: content.to_string(),
            content: content.to_string(),
            path,
            ..Default::default()
        }
    }

    pub fn set_content(mut self, content: &str) -> Self {
        self.nodes = markdown_parser::from_str(content);
        self.content_original = content.to_string();
        self.content = content.to_string();
        self.update_text_buffer();
        self
    }

    pub fn set_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn exit_insert(mut self) -> Self {
        self.intermediate_save();
        self
    }

    fn intermediate_save(&mut self) {
        if let Some(node) = self.nodes().get(self.current_row) {
            let start = node.source_range.start;
            let end = node.source_range.end;

            let str_start = &self.content_slice(..start.saturating_sub(1));
            let str_end = &self.content_slice(end..);

            let modified_str = self.text_buffer().to_string();

            let complete_modified_content = [str_start, modified_str.as_str(), str_end].join("\n");

            if self.content != complete_modified_content {
                self.nodes = markdown_parser::from_str(&complete_modified_content);
                self.content = complete_modified_content;
                self.update_text_buffer();
            }

            self.modified = self.content != self.content_original;
        }
    }

    pub fn delete_char(mut self) -> Self {
        let (row, col) = self.text_buffer.cursor();

        if row == 0 && col == 0 && self.text_buffer().to_string().trim().is_empty() {
            self.intermediate_save();
        } else if row == 0 && col == 0 && self.current_row != 0 {
            let current_row = self.current_row;
            let content = self.content.clone();
            let mut nodes = self.nodes_as_mut().to_vec();

            if let Some(current_node_range_end) =
                nodes.get(current_row).map(|node| node.source_range.end)
            {
                if let Some(prev_node) = nodes.get_mut(current_row - 1) {
                    let content = &content[prev_node.source_range.clone()];
                    prev_node.source_range = prev_node.source_range.start..current_node_range_end;
                    self.update_text_buffer_content(content);
                    nodes.remove(current_row);
                    self.nodes = nodes;
                    self.current_row = current_row.saturating_sub(1);
                    self.dirty = true;
                }
            }
        } else {
            self.dirty = true;
            self.text_buffer.edit(Input {
                key: tui_textarea::Key::Backspace,
                ctrl: false,
                alt: false,
                shift: false,
            });
        }

        self
    }

    pub fn edit(mut self, input: Input) -> Self {
        self.text_buffer.edit(input);
        if self.text_buffer.is_modified() {
            self.dirty = true;
        }
        self
    }

    pub fn cursor_up(mut self) -> Self {
        let (row, _) = self.text_buffer.cursor();
        if row == 0 {
            if self.dirty {
                self.intermediate_save();
                self.dirty = false;
            }

            if self.current_row == 0 {
                return self;
            }

            self.current_row = self.current_row.saturating_sub(1);
            self.update_text_buffer();
            self.text_buffer.cursor_move(CursorMove::Bottom);
        } else {
            self.text_buffer.cursor_move(CursorMove::Up);
        }

        self
    }

    pub fn cursor_left(mut self) -> Self {
        self.text_buffer.cursor_move(CursorMove::Left);
        self
    }

    pub fn cursor_right(mut self) -> Self {
        self.text_buffer.cursor_move(CursorMove::Right);
        self
    }

    pub fn cursor_move_col(mut self, cursor_move_col: i32) -> Self {
        self.text_buffer.cursor_move((0, cursor_move_col).into());
        self
    }

    pub fn cursor_word_forward(mut self) -> Self {
        self.text_buffer.cursor_move(CursorMove::WordForward);
        self
    }

    pub fn cursor_word_backward(mut self) -> Self {
        self.text_buffer.cursor_move(CursorMove::WordBackward);
        self
    }

    pub fn cursor_down(mut self) -> Self {
        let (row, _) = self.text_buffer.cursor();
        if row < self.text_buffer.lines().len().saturating_sub(1) {
            self.text_buffer.cursor_move(CursorMove::Down);
            return self;
        } else {
            if self.dirty {
                self.intermediate_save();
                self.dirty = false;
            }

            let nodes_amount = self.nodes.len();

            if self.current_row == nodes_amount.saturating_sub(1) {
                return self;
            }

            // let nodes = markdown_parser::from_str(self.raw());
            // let diff = nodes_amount.abs_diff(nodes.len());
            // self.nodes = nodes;

            self.current_row = self
                .current_row
                .saturating_add(1)
                // .saturating_add(diff)
                .min(self.nodes.len().saturating_sub(1));

            self.update_text_buffer();
            self.text_buffer.cursor_move(CursorMove::Top);
        }

        self
    }

    pub fn save(mut self) -> Self {
        if !self.modified {
            return self;
        }

        match self.save_modified_to_file() {
            Ok(_) => self,
            Err(_err) => Self {
                // TODO: Display error messages
                // error_message: Some(format!("Failed to save file: {}", err)),
                ..self
            },
        }
    }

    fn save_modified_to_file(&mut self) -> io::Result<()> {
        let mut file = File::create(&self.path)?;
        file.write_all(self.content.as_bytes())?;
        self.modified = false;
        Ok(())
    }

    pub fn scroll_up(self, amount: usize) -> Self {
        let new_position = self.scrollbar.position.saturating_sub(amount);
        let new_state = self.scrollbar.state.position(new_position);

        // TODO: Advance cursor and try to keep the cursor centered.
        // Look for inspiration from the explorer module list scrolling where the list item is kept
        // in the center, if it is possible. This should be used to scroll the view instead of
        // directly changing the scrollbar in this function.

        Self {
            scrollbar: Scrollbar {
                state: new_state,
                position: new_position,
            },
            ..self
        }
    }

    pub fn scroll_down(self, amount: usize) -> Self {
        let new_position = self.scrollbar.position.saturating_add(amount);
        let new_state = self.scrollbar.state.position(new_position);

        Self {
            scrollbar: Scrollbar {
                state: new_state,
                position: new_position,
            },
            ..self
        }
    }

    pub fn set_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn text_buffer(&self) -> &TextBuffer<'text_buffer> {
        &self.text_buffer
    }

    pub fn text_buffer_as_mut(&mut self) -> &mut TextBuffer<'text_buffer> {
        self.text_buffer.as_mut()
    }

    pub fn set_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn update_text_buffer_content(&mut self, content: &str) {
        let text_buffer_content = self.text_buffer().to_string();
        let (_, col) = self.text_buffer.cursor();
        self.text_buffer = TextBuffer::from(format!("{content}\n{text_buffer_content}"))
            .with_cursor_position((content.lines().count() + 1, col));
    }

    pub fn update_text_buffer(&mut self) {
        if let Some(node) = self.nodes().get(self.current_row) {
            let node_content = self.content_slice(node.source_range.clone());
            self.text_buffer =
                TextBuffer::from(node_content).with_cursor_position(self.text_buffer.cursor());
        }
    }

    pub fn reset(self) -> Self {
        Self {
            mode: self.mode,
            ..EditorState::default()
        }
    }
}
