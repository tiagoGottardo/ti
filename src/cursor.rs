const CURSOR_BLOCK: usize = 2;
const CURSOR_UNDERLINE: usize = 4;
const CURSOR_BAR: usize = 6;

use std::cmp::{max, min};

use crate::{
    app::Mode,
    document::{Document, Pos},
    render_buffer::{LEFT_SPACE_SIZE, TOP_SPACE_SIZE},
    viewport::Viewport,
};

#[derive(Copy, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    pub fn build(&self, doc: &Document, viewport: &Viewport, mode: Mode) -> String {
        use Mode::*;

        let mut building = String::new();

        let col = min(self.col, doc.col_bound(self.row, mode));

        building.push_str(&format!(
            "\x1b[{};{}H",
            self.row - viewport.top_row + 1 + TOP_SPACE_SIZE,
            col - viewport.left_column + 1 + LEFT_SPACE_SIZE
        ));

        let mode = match mode {
            Normal | Visual(_) => CURSOR_BLOCK,
            Replace | Delete | Copy => CURSOR_UNDERLINE,
            Insert => CURSOR_BAR,
        };

        building.push_str(&format!("\x1b[{} q", mode));

        building
    }

    pub fn bound_col(&mut self, doc: &Document, mode: Mode) -> &mut Self {
        self.col = max(min(self.col, doc.col_bound(self.row, mode)), 0);

        self
    }

    pub fn bound_row(&mut self, doc: &Document) -> &mut Self {
        self.row = max(min(self.row, doc.row_bound()), 0);

        self
    }

    pub fn left(&mut self) -> &mut Self {
        if self.col > 0 {
            self.col -= 1;
        }

        self
    }

    pub fn right(&mut self, doc: &Document, mode: Mode) -> &mut Self {
        self.col += 1;
        self.bound_col(doc, mode);

        self
    }

    pub fn down(&mut self, doc: &Document) -> &mut Self {
        self.row += 1;
        self.bound_row(doc);

        self
    }

    pub fn up(&mut self) -> &mut Self {
        if self.row > 0 {
            self.row -= 1;
        }

        self
    }

    pub fn go_to_first_line(&mut self) -> &mut Self {
        self.row = 0;

        self
    }

    pub fn go_to_last_char(&mut self, doc: &Document) -> &mut Self {
        self.row = doc.row_bound();
        self.col = doc.col_bound(self.row, Mode::Normal);

        self
    }

    pub fn go_to_start_of_line(&mut self, doc: &Document) -> &mut Self {
        self.col = doc.lines[self.row]
            .chars()
            .enumerate()
            .find(|(_, ch)| *ch != ' ')
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        self
    }

    pub fn go_to_end_of_line(&mut self, doc: &Document, mode: Mode) -> &mut Self {
        self.col = doc.col_bound(self.row, mode);

        self
    }

    pub fn to_pos(&self) -> Pos {
        Pos {
            row: self.row,
            col: self.col,
        }
    }

    pub fn go_to_pos(&mut self, pos: Pos) -> &mut Self {
        self.row = pos.row;
        self.col = pos.col;

        self
    }
}
